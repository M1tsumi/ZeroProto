fn main() -> Result<(), Box<dyn std::error::Error>> {
    zeroproto_compiler::build()?;
    Ok(())
}
