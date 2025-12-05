//! ZeroProto CLI - Command-line interface for ZeroProto schema compilation

use clap::{ArgAction, Args, Parser, Subcommand};
use color_eyre::eyre::{eyre, Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use zeroproto_compiler::{
    self as compiler,
    ast::{DefaultValue, Enum, Field, FieldType, Message, ScalarType, Schema, SchemaItem},
};

#[derive(Parser)]
#[command(name = "zeroproto")]
#[command(about = "ZeroProto schema compiler and code generator")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Args, Debug, Clone)]
struct SchemaFilterArgs {
    /// Glob patterns to include (matched relative to the input path)
    #[arg(long = "include", action = ArgAction::Append)]
    include: Vec<String>,
    /// Glob patterns to exclude (matched relative to the input path)
    #[arg(long = "exclude", action = ArgAction::Append)]
    exclude: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct CompiledSchemaFilters {
    include: Option<GlobSet>,
    exclude: Option<GlobSet>,
}

impl SchemaFilterArgs {
    fn build(&self) -> Result<CompiledSchemaFilters> {
        let include = if self.include.is_empty() {
            None
        } else {
            let mut builder = GlobSetBuilder::new();
            for pattern in &self.include {
                builder.add(Glob::new(pattern)?);
            }
            Some(builder.build()?)
        };

        let exclude = if self.exclude.is_empty() {
            None
        } else {
            let mut builder = GlobSetBuilder::new();
            for pattern in &self.exclude {
                builder.add(Glob::new(pattern)?);
            }
            Some(builder.build()?)
        };

        Ok(CompiledSchemaFilters { include, exclude })
    }

    fn should_include(
        &self,
        compiled: &CompiledSchemaFilters,
        path: &Path,
        input_root: &Path,
    ) -> bool {
        let relative = path.strip_prefix(input_root).unwrap_or(path);
        if let Some(include) = &compiled.include {
            if !include.is_match(relative) {
                return false;
            }
        }
        if let Some(exclude) = &compiled.exclude {
            if exclude.is_match(relative) {
                return false;
            }
        }
        true
    }

    fn summary<'a>(&self, included: &[&'a Path], skipped: &[&'a Path]) {
        if !included.is_empty() {
            println!("📦 Included {} schemas:", included.len());
            for path in included {
                println!("  ✅ {}", path.display());
            }
        }
        if !skipped.is_empty() {
            println!("🚫 Skipped {} schemas:", skipped.len());
            for path in skipped {
                println!("  ⚠️ {}", path.display());
            }
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Compile schema files to Rust code
    Compile {
        /// Schema file or directory to compile
        #[arg(short, long)]
        input: PathBuf,
        /// Output directory for generated code
        #[arg(short, long, default_value = "src/generated")]
        output: PathBuf,
        /// Run in verbose mode
        #[arg(short, long)]
        verbose: bool,
        /// Schema filtering options
        #[command(flatten)]
        filters: SchemaFilterArgs,
    },
    /// Watch schema files for changes and recompile automatically
    Watch {
        /// Schema directory to watch
        #[arg(short, long)]
        input: PathBuf,
        /// Output directory for generated code
        #[arg(short, long, default_value = "src/generated")]
        output: PathBuf,
        /// Run in verbose mode
        #[arg(short, long)]
        verbose: bool,
        /// Schema filtering options
        #[command(flatten)]
        filters: SchemaFilterArgs,
    },
    /// Validate schema files without generating code
    Check {
        /// Schema file or directory to check
        #[arg(short, long)]
        input: PathBuf,
        /// Run in verbose mode
        #[arg(short, long)]
        verbose: bool,
        /// Schema filtering options
        #[command(flatten)]
        filters: SchemaFilterArgs,
    },
    /// Inspect schema structure and statistics without generating code
    Inspect {
        /// Schema file or directory to inspect
        #[arg(short, long)]
        input: PathBuf,
        /// Show additional details
        #[arg(short, long)]
        verbose: bool,
        /// Schema filtering options
        #[command(flatten)]
        filters: SchemaFilterArgs,
    },
    /// Create a new ZeroProto project
    Init {
        /// Project name
        #[arg(short, long)]
        name: String,
        /// Create project in current directory
        #[arg(short, long)]
        current_dir: bool,
    },
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Compile {
            input,
            output,
            verbose,
            filters,
        } => {
            let compiled = filters.build()?;
            compile_schemas(&input, &output, verbose, &filters, &compiled)
        }
        Commands::Watch {
            input,
            output,
            verbose,
            filters,
        } => {
            let compiled = filters.build()?;
            watch_schemas(&input, &output, verbose, &filters, &compiled)
        }
        Commands::Check {
            input,
            verbose,
            filters,
        } => {
            let compiled = filters.build()?;
            check_schemas(&input, verbose, &filters, &compiled)
        }
        Commands::Inspect {
            input,
            verbose,
            filters,
        } => {
            let compiled = filters.build()?;
            inspect_schemas(&input, verbose, &filters, &compiled)
        }
        Commands::Init { name, current_dir } => init_project(&name, current_dir),
    }
}

/// Compile schema files
fn compile_schemas(
    input: &Path,
    output: &Path,
    verbose: bool,
    filters: &SchemaFilterArgs,
    compiled: &CompiledSchemaFilters,
) -> Result<()> {
    if input.is_file() {
        if !should_process_file(input, filters, compiled) {
            println!("🚫 Skipping {} (filtered out)", input.display());
            return Ok(());
        }

        if verbose {
            println!("Compiling schema file: {}", input.display());
        }
        compiler::compile(input, output)
            .with_context(|| format!("Failed to compile schema file: {}", input.display()))?;

        if verbose {
            println!("Generated code in: {}", output.display());
        }
    } else if input.is_dir() {
        let (schema_files, skipped) = collect_schema_files(input, filters, compiled)?;

        if schema_files.is_empty() {
            if skipped.is_empty() {
                eprintln!("No .zp schema files found in: {}", input.display());
            } else {
                eprintln!("All schema files in {} were filtered out.", input.display());
            }
            return Ok(());
        }

        if verbose {
            let included_refs: Vec<&Path> = schema_files.iter().map(|p| p.as_path()).collect();
            let skipped_refs: Vec<&Path> = skipped.iter().map(|p| p.as_path()).collect();
            filters.summary(&included_refs, &skipped_refs);
        }

        compiler::compile_multiple(&schema_files, output.to_path_buf())
            .with_context(|| format!("Failed to compile schemas in: {}", input.display()))?;

        if verbose {
            println!("Generated code in: {}", output.display());
        }
    } else {
        return Err(eyre!("Input path does not exist: {}", input.display()));
    }

    println!("✅ Compilation successful!");
    Ok(())
}

/// Watch schema files for changes
fn watch_schemas(
    input: &Path,
    output: &Path,
    verbose: bool,
    filters: &SchemaFilterArgs,
    compiled: &CompiledSchemaFilters,
) -> Result<()> {
    if !input.is_dir() {
        return Err(eyre!(
            "Watch input must be a directory: {}",
            input.display()
        ));
    }

    println!("👀 Watching schema files in: {}", input.display());
    println!("📁 Output directory: {}", output.display());
    println!("Press Ctrl+C to stop watching...");

    // Initial compilation
    compile_schemas(input, output, verbose, filters, compiled)?;

    // Set up file watcher
    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(input, RecursiveMode::Recursive)?;

    loop {
        match rx.recv() {
            Ok(Ok(event)) => match event.kind {
                EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
                    let mut touched = false;
                    for path in &event.paths {
                        if should_process_path(path, input, filters, compiled) {
                            println!("🔄 Detected schema change in: {}", path.display());
                            touched = true;
                            break;
                        }
                    }

                    if touched {
                        match compile_schemas(input, output, verbose, filters, compiled) {
                            Ok(()) => println!("✅ Recompilation successful!"),
                            Err(e) => eprintln!("❌ Recompilation failed: {}", e),
                        }
                    }
                }
                _ => {}
            },
            Ok(Err(e)) => eprintln!("Watch error: {:?}", e),
            Err(e) => {
                eprintln!("Channel error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

/// Check schema files without generating code
fn check_schemas(
    input: &Path,
    verbose: bool,
    filters: &SchemaFilterArgs,
    compiled: &CompiledSchemaFilters,
) -> Result<()> {
    if input.is_file() {
        if !should_process_file(input, filters, compiled) {
            println!("🚫 Skipping {} (filtered out)", input.display());
            return Ok(());
        }

        if verbose {
            println!("Checking schema file: {}", input.display());
        }

        let content = fs::read_to_string(input)
            .with_context(|| format!("Failed to read schema file: {}", input.display()))?;

        compiler::parse(&content)
            .with_context(|| format!("Schema validation failed for: {}", input.display()))?;

        println!("✅ Schema file is valid: {}", input.display());
    } else if input.is_dir() {
        let (schema_files, skipped) = collect_schema_files(input, filters, compiled)?;

        if schema_files.is_empty() {
            if skipped.is_empty() {
                eprintln!("No .zp schema files found in: {}", input.display());
            } else {
                eprintln!("All schema files in {} were filtered out.", input.display());
            }
            return Ok(());
        }

        if verbose {
            println!(
                "Checking {} schema files in: {}",
                schema_files.len(),
                input.display()
            );
            for file in &schema_files {
                println!("  Checking: {}", file.display());
            }
            if !skipped.is_empty() {
                println!("Skipped {} files due to filters.", skipped.len());
            }
        }

        for file in &schema_files {
            let content = fs::read_to_string(file)
                .with_context(|| format!("Failed to read schema file: {}", file.display()))?;

            compiler::parse(&content)
                .with_context(|| format!("Schema validation failed for: {}", file.display()))?;
        }

        println!("✅ All {} schema files are valid!", schema_files.len());
    } else {
        return Err(eyre!("Input path does not exist: {}", input.display()));
    }

    Ok(())
}

/// Inspect schema structure and print statistics
fn inspect_schemas(
    input: &Path,
    verbose: bool,
    filters: &SchemaFilterArgs,
    compiled: &CompiledSchemaFilters,
) -> Result<()> {
    let mut schema_files = Vec::new();
    let mut skipped = Vec::new();

    if input.is_file() {
        if should_process_file(input, filters, compiled) {
            schema_files.push(input.to_path_buf());
        } else {
            println!("🚫 Skipping {} (filtered out)", input.display());
            return Ok(());
        }
    } else if input.is_dir() {
        let (included, skipped_paths) = collect_schema_files(input, filters, compiled)?;
        schema_files = included;
        skipped = skipped_paths;
    } else {
        return Err(eyre!("Input path does not exist: {}", input.display()));
    }

    if schema_files.is_empty() {
        if skipped.is_empty() {
            eprintln!("No .zp schema files found in: {}", input.display());
        } else {
            eprintln!("All schema files in {} were filtered out.", input.display());
        }
        return Ok(());
    }

    if verbose && !skipped.is_empty() {
        let included_refs: Vec<&Path> = schema_files.iter().map(|p| p.as_path()).collect();
        let skipped_refs: Vec<&Path> = skipped.iter().map(|p| p.as_path()).collect();
        filters.summary(&included_refs, &skipped_refs);
    }

    let mut totals = AggregateStats::default();

    for file in &schema_files {
        let content = fs::read_to_string(file)
            .with_context(|| format!("Failed to read schema file: {}", file.display()))?;
        let schema = compiler::parse(&content)
            .with_context(|| format!("Failed to parse schema: {}", file.display()))?;
        let insight = analyze_schema(&schema);

        totals.files += 1;
        totals.messages += insight.messages.len();
        totals.enums += insight.enums.len();
        totals.fields += insight.total_fields;
        totals.optional_fields += insight.optional_fields;
        totals.defaulted_fields += insight.defaulted_fields;
        totals.vector_fields += insight.vector_fields;

        println!("\n📄 {}", file.display());
        println!(
            "   Messages: {} | Enums: {}",
            insight.messages.len(),
            insight.enums.len()
        );
        println!(
            "   Fields: {} (optional {}, defaults {}, vectors {})",
            insight.total_fields,
            insight.optional_fields,
            insight.defaulted_fields,
            insight.vector_fields
        );

        if verbose {
            for msg in &insight.messages {
                println!(
                    "     • msg {} — fields: {}, optional: {}, defaults: {}, vectors: {}",
                    msg.name,
                    msg.field_count,
                    msg.optional_fields,
                    msg.defaulted_fields,
                    msg.vector_fields
                );
            }
            for en in &insight.enums {
                println!("     • enum {} — variants: {}", en.name, en.variant_count);
            }
        }
    }

    println!("\n📊 Inspection Summary");
    println!("   Files: {}", totals.files);
    println!("   Messages: {}", totals.messages);
    println!("   Enums: {}", totals.enums);
    println!(
        "   Fields: {} (optional {}, defaults {}, vectors {})",
        totals.fields, totals.optional_fields, totals.defaulted_fields, totals.vector_fields
    );

    Ok(())
}

#[derive(Default)]
struct AggregateStats {
    files: usize,
    messages: usize,
    enums: usize,
    fields: usize,
    optional_fields: usize,
    defaulted_fields: usize,
    vector_fields: usize,
}

struct SchemaInsight {
    messages: Vec<MessageInsight>,
    enums: Vec<EnumInsight>,
    total_fields: usize,
    optional_fields: usize,
    defaulted_fields: usize,
    vector_fields: usize,
}

struct MessageInsight {
    name: String,
    field_count: usize,
    optional_fields: usize,
    defaulted_fields: usize,
    vector_fields: usize,
}

struct EnumInsight {
    name: String,
    variant_count: usize,
}

fn analyze_schema(schema: &Schema) -> SchemaInsight {
    let mut messages = Vec::new();
    let mut enums = Vec::new();
    let mut total_fields = 0;
    let mut optional_fields = 0;
    let mut defaulted_fields = 0;
    let mut vector_fields = 0;

    for item in &schema.items {
        match item {
            SchemaItem::Message(message) => {
                let mut msg_fields = 0;
                let mut msg_optional = 0;
                let mut msg_defaulted = 0;
                let mut msg_vectors = 0;

                for field in &message.fields {
                    msg_fields += 1;
                    if field.optional {
                        msg_optional += 1;
                    }
                    if field.default_value.is_some() {
                        msg_defaulted += 1;
                    }
                    if matches!(field.field_type, FieldType::Vector(_)) {
                        msg_vectors += 1;
                    }
                }

                total_fields += msg_fields;
                optional_fields += msg_optional;
                defaulted_fields += msg_defaulted;
                vector_fields += msg_vectors;

                messages.push(MessageInsight {
                    name: message.name.clone(),
                    field_count: msg_fields,
                    optional_fields: msg_optional,
                    defaulted_fields: msg_defaulted,
                    vector_fields: msg_vectors,
                });
            }
            SchemaItem::Enum(en) => {
                enums.push(EnumInsight {
                    name: en.name.clone(),
                    variant_count: en.variants.len(),
                });
            }
        }
    }

    SchemaInsight {
        messages,
        enums,
        total_fields,
        optional_fields,
        defaulted_fields,
        vector_fields,
    }
}

/// Initialize a new ZeroProto project
fn init_project(name: &str, current_dir: bool) -> Result<()> {
    let project_dir = if current_dir {
        PathBuf::from(".")
    } else {
        PathBuf::from(name)
    };

    if project_dir.exists() && !current_dir {
        return Err(color_eyre::eyre::eyre!(
            "Directory already exists: {}",
            project_dir.display()
        ));
    }

    // Create project directory
    if !current_dir {
        std::fs::create_dir(&project_dir)?;
        println!("📁 Created project directory: {}", project_dir.display());
    }

    // Create Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.2.0"
edition = "2024"

[dependencies]
zeroproto = "0.2.0"

[build-dependencies]
zeroproto-compiler = "0.2.0"
"#,
        name
    );

    std::fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;
    println!("📄 Created Cargo.toml");

    // Create build.rs
    let build_rs = r#"fn main() -> Result<(), Box<dyn std::error::Error>> {
    zeroproto_compiler::build()?;
    Ok(())
}
"#;

    std::fs::write(project_dir.join("build.rs"), build_rs)?;
    println!("📄 Created build.rs");

    // Create src directory and main.rs
    std::fs::create_dir_all(project_dir.join("src"))?;
    let main_rs = r#"fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Your ZeroProto code will go here
    println!("Hello, ZeroProto!");
    Ok(())
}
"#;

    std::fs::write(project_dir.join("src/main.rs"), main_rs)?;
    println!("📄 Created src/main.rs");

    // Create schemas directory and example schema
    std::fs::create_dir_all(project_dir.join("schemas"))?;
    let example_schema = format!(
        r#"// Example schema for {} project
message User {{
    id: u64;
    username: string;
    email: string;
    age: u8;
}}
"#,
        name
    );

    std::fs::write(project_dir.join("schemas/example.zp"), example_schema)?;
    println!("📄 Created schemas/example.zp");

    // Create README.md
    let readme = format!(
        r#"# {}

A ZeroProto project.

## Usage

1. Edit your schema files in the `schemas/` directory
2. Run `cargo build` to generate Rust code
3. Use the generated types in your application

## Generated Code

The Rust code generated from your schemas will be available in `src/generated/`.

## Example

```rust
mod generated;
use generated::example::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {{
    // Create a user
    let mut builder = UserBuilder::new();
    builder.set_id(123);
    builder.set_username("alice");
    builder.set_email("alice@example.com");
    builder.set_age(25);
    
    let data = builder.finish();
    
    // Read the user (zero-copy!)
    let user = UserReader::from_slice(&data)?;
    println!("User: {{}}", user.username());
    
    Ok(())
}}
```
"#,
        name
    );

    std::fs::write(project_dir.join("README.md"), readme)?;
    println!("📄 Created README.md");

    // Create .gitignore
    let gitignore = r#"# Generated code
src/generated/

# Rust
target/
Cargo.lock
**/*.rs.bk

# IDE
.vscode/
.idea/
"#;

    std::fs::write(project_dir.join(".gitignore"), gitignore)?;
    println!("📄 Created .gitignore");

    println!("\n🎉 Project '{}' initialized successfully!", name);
    if !current_dir {
        println!("   cd {}", name);
    }
    println!("   cargo build");

    Ok(())
}

/// Find all .zp schema files in a directory
fn collect_schema_files(
    dir: &Path,
    filters: &SchemaFilterArgs,
    compiled: &CompiledSchemaFilters,
) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let mut included = Vec::new();
    let mut skipped = Vec::new();
    collect_schema_files_inner(dir, dir, filters, compiled, &mut included, &mut skipped)?;

    included.sort();
    skipped.sort();

    Ok((included, skipped))
}

fn collect_schema_files_inner(
    root: &Path,
    current: &Path,
    filters: &SchemaFilterArgs,
    compiled: &CompiledSchemaFilters,
    included: &mut Vec<PathBuf>,
    skipped: &mut Vec<PathBuf>,
) -> Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                if std::env::var("ZEROPROTO_CLI_STRICT").is_ok() {
                    return Err(err.into());
                }
                continue;
            }
        };
        let path = entry.path();

        if path.is_file() && path.extension().map_or(false, |ext| ext == "zp") {
            if should_process_path(&path, root, filters, compiled) {
                included.push(path);
            } else {
                skipped.push(path);
            }
        } else if path.is_dir() {
            if let Err(err) =
                collect_schema_files_inner(root, &path, filters, compiled, included, skipped)
            {
                if std::env::var("ZEROPROTO_CLI_STRICT").is_ok() {
                    return Err(err);
                }
                continue;
            }
        }
    }

    Ok(())
}

fn should_process_file(
    path: &Path,
    filters: &SchemaFilterArgs,
    compiled: &CompiledSchemaFilters,
) -> bool {
    let root = path.parent().unwrap_or_else(|| Path::new("."));
    should_process_path(path, root, filters, compiled)
}

fn should_process_path(
    path: &Path,
    root: &Path,
    filters: &SchemaFilterArgs,
    compiled: &CompiledSchemaFilters,
) -> bool {
    if path.extension().map_or(true, |ext| ext != "zp") {
        return false;
    }
    filters.should_include(compiled, path, root)
}
