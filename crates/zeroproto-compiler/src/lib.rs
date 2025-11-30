//! ZeroProto schema compiler and code generator

pub mod ast;
pub mod codegen;
pub mod ir;
pub mod parser;
pub mod primitives;
pub mod validator;

// Re-export commonly used functions
pub use parser::parse;

use thiserror::Error;
use std::path::{Path, PathBuf};

/// Result type for compiler operations
pub type Result<T> = std::result::Result<T, CompilerError>;

/// Errors that can occur during compilation
#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Code generation error: {0}")]
    Codegen(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
}

/// Compile a ZeroProto schema file to Rust code
pub fn compile<P: AsRef<Path>>(schema_path: P, output_dir: P) -> Result<()> {
    let schema_path = schema_path.as_ref();
    let output_dir = output_dir.as_ref();
    
    // Read schema file
    let schema_content = std::fs::read_to_string(schema_path)
        .map_err(|_| CompilerError::FileNotFound(schema_path.to_string_lossy().to_string()))?;
    
    // Parse schema
    let ast = parser::parse(&schema_content)?;
    
    // Validate AST
    validator::validate(&ast)?;
    
    // Convert to IR
    let ir = ir::lower_ast(&ast);
    
    // Generate code
    let generated_code = codegen::generate_rust_code(&ir)?;
    
    // Create output directory if it doesn't exist
    std::fs::create_dir_all(output_dir)?;
    
    // Write generated code
    let output_path = output_dir.join(format!(
        "{}.rs",
        schema_path.file_stem().unwrap().to_string_lossy()
    ));
    std::fs::write(output_path, generated_code)?;
    
    Ok(())
}

/// Compile multiple schema files
pub fn compile_multiple<P: AsRef<Path>>(schema_files: &[P], output_dir: P) -> Result<()> {
    for schema_file in schema_files {
        compile(schema_file, &output_dir)?;
    }
    
    // Generate mod.rs file
    generate_mod_rs(output_dir)?;
    
    Ok(())
}

/// Generate mod.rs file for the output directory
fn generate_mod_rs<P: AsRef<Path>>(output_dir: P) -> Result<()> {
    let output_dir = output_dir.as_ref();
    
    let mut mod_content = String::new();
    
    // Read all .rs files in the directory
    for entry in std::fs::read_dir(output_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().map_or(false, |ext| ext == "rs") && 
           path.file_name().map_or(false, |name| name != "mod.rs") {
            if let Some(stem) = path.file_stem() {
                mod_content.push_str(&format!("pub mod {};\n", stem.to_string_lossy()));
            }
        }
    }
    
    let mod_path = output_dir.join("mod.rs");
    std::fs::write(mod_path, mod_content)?;
    
    Ok(())
}

/// Convenience function for build.rs scripts
pub fn build() -> Result<()> {
    // Look for schema files in common locations
    let schema_dirs = ["schemas", "src/schemas", "."];
    
    for dir in &schema_dirs {
        if Path::new(dir).exists() {
            let schema_files: Vec<_> = std::fs::read_dir(dir)?
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "zp") {
                        Some(path)
                    } else {
                        None
                    }
                })
                .collect();
            
            if !schema_files.is_empty() {
                return compile_multiple(&schema_files, PathBuf::from("src/generated"));
            }
        }
    }
    
    Err(CompilerError::FileNotFound("No schema files found".to_string()))
}
