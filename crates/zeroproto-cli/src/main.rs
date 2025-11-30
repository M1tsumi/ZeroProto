//! ZeroProto CLI - Command-line interface for ZeroProto schema compilation

use clap::{Parser, Subcommand};
use color_eyre::eyre::{Context, Result};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use zeroproto_compiler as compiler;

#[derive(Parser)]
#[command(name = "zeroproto")]
#[command(about = "ZeroProto schema compiler and code generator")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
    },
    /// Validate schema files without generating code
    Check {
        /// Schema file or directory to check
        #[arg(short, long)]
        input: PathBuf,
        /// Run in verbose mode
        #[arg(short, long)]
        verbose: bool,
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
        Commands::Compile { input, output, verbose } => {
            compile_schemas(&input, &output, verbose)
        }
        Commands::Watch { input, output, verbose } => {
            watch_schemas(&input, &output, verbose)
        }
        Commands::Check { input, verbose } => {
            check_schemas(&input, verbose)
        }
        Commands::Init { name, current_dir } => {
            init_project(&name, current_dir)
        }
    }
}

/// Compile schema files
fn compile_schemas(input: &Path, output: &Path, verbose: bool) -> Result<()> {
    if input.is_file() {
        // Compile single file
        if verbose {
            println!("Compiling schema file: {}", input.display());
        }
        compiler::compile(input, output)
            .with_context(|| format!("Failed to compile schema file: {}", input.display()))?;
        
        if verbose {
            println!("Generated code in: {}", output.display());
        }
    } else if input.is_dir() {
        // Compile all .zp files in directory
        let schema_files = find_schema_files(input)?;
        
        if schema_files.is_empty() {
            eprintln!("No .zp schema files found in: {}", input.display());
            return Ok(());
        }
        
        if verbose {
            println!("Found {} schema files in: {}", schema_files.len(), input.display());
            for file in &schema_files {
                println!("  {}", file.display());
            }
        }
        
        compiler::compile_multiple(&schema_files, output.to_path_buf())
            .with_context(|| format!("Failed to compile schemas in: {}", input.display()))?;
        
        if verbose {
            println!("Generated code in: {}", output.display());
        }
    } else {
        return Err(color_eyre::eyre::eyre!("Input path does not exist: {}", input.display()));
    }
    
    println!("✅ Compilation successful!");
    Ok(())
}

/// Watch schema files for changes
fn watch_schemas(input: &Path, output: &Path, verbose: bool) -> Result<()> {
    if !input.is_dir() {
        return Err(color_eyre::eyre::eyre!("Watch input must be a directory: {}", input.display()));
    }
    
    println!("👀 Watching schema files in: {}", input.display());
    println!("📁 Output directory: {}", output.display());
    println!("Press Ctrl+C to stop watching...");
    
    // Initial compilation
    compile_schemas(input, output, verbose)?;
    
    // Set up file watcher
    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(input, RecursiveMode::Recursive)?;
    
    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                if let EventKind::Modify(_) = event.kind {
                    for path in &event.paths {
                        if path.extension().map_or(false, |ext| ext == "zp") {
                            println!("🔄 Detected changes in: {}", path.display());
                            match compile_schemas(input, output, verbose) {
                                Ok(()) => println!("✅ Recompilation successful!"),
                                Err(e) => eprintln!("❌ Recompilation failed: {}", e),
                            }
                            break;
                        }
                    }
                }
            }
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
fn check_schemas(input: &Path, verbose: bool) -> Result<()> {
    if input.is_file() {
        // Check single file
        if verbose {
            println!("Checking schema file: {}", input.display());
        }
        
        let content = std::fs::read_to_string(input)
            .with_context(|| format!("Failed to read schema file: {}", input.display()))?;
        
        compiler::parse(&content)
            .with_context(|| format!("Schema validation failed for: {}", input.display()))?;
        
        println!("✅ Schema file is valid: {}", input.display());
    } else if input.is_dir() {
        // Check all .zp files in directory
        let schema_files = find_schema_files(input)?;
        
        if schema_files.is_empty() {
            eprintln!("No .zp schema files found in: {}", input.display());
            return Ok(());
        }
        
        if verbose {
            println!("Checking {} schema files in: {}", schema_files.len(), input.display());
        }
        
        for file in &schema_files {
            if verbose {
                println!("  Checking: {}", file.display());
            }
            
            let content = std::fs::read_to_string(file)
                .with_context(|| format!("Failed to read schema file: {}", file.display()))?;
            
            compiler::parse(&content)
                .with_context(|| format!("Schema validation failed for: {}", file.display()))?;
        }
        
        println!("✅ All {} schema files are valid!", schema_files.len());
    } else {
        return Err(color_eyre::eyre::eyre!("Input path does not exist: {}", input.display()));
    }
    
    Ok(())
}

/// Initialize a new ZeroProto project
fn init_project(name: &str, current_dir: bool) -> Result<()> {
    let project_dir = if current_dir {
        PathBuf::from(".")
    } else {
        PathBuf::from(name)
    };
    
    if project_dir.exists() && !current_dir {
        return Err(color_eyre::eyre::eyre!("Directory already exists: {}", project_dir.display()));
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
fn find_schema_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "zp") {
            files.push(path);
        } else if path.is_dir() {
            // Recursively search subdirectories
            match find_schema_files(&path) {
                Ok(mut sub_files) => files.append(&mut sub_files),
                Err(_) => continue, // Skip directories we can't read
            }
        }
    }
    
    files.sort();
    Ok(files)
}
