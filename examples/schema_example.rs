//! Schema-based usage example for ZeroProto
//! This demonstrates the full workflow with schema compilation

use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ZeroProto Schema Example");
    println!("=========================");
    
    // Create a temporary directory for our example
    let temp_dir = std::env::temp_dir().join("zeroproto_example");
    fs::create_dir_all(&temp_dir)?;
    fs::create_dir_all(temp_dir.join("schemas"))?;
    fs::create_dir_all(temp_dir.join("src"))?;
    
    // Create a schema file
    let schema_content = r#"
enum Status {
    Active = 1;
    Inactive = 2;
    Pending = 3;
}

message User {
    user_id: u64;
    username: string;
    email: string;
    age: u8;
    status: Status;
    tags: [string];
    profile: UserProfile;
}

message UserProfile {
    bio: string;
    avatar_url: string;
    settings: UserSettings;
}

message UserSettings {
    theme: Theme;
    notifications_enabled: bool;
    max_friends: u32;
}

enum Theme {
    Light = 0;
    Dark = 1;
    Auto = 2;
}
"#;
    
    let schema_path = temp_dir.join("schemas/user.zp");
    fs::write(&schema_path, schema_content)?;
    
    // Create build.rs
    let build_rs_content = r#"
fn main() -> Result<(), Box<dyn std::error::Error>> {
    zeroproto_compiler::compile_multiple(
        &["schemas/user.zp"],
        "src/generated"
    )?;
    Ok(())
}
"#;
    
    fs::write(temp_dir.join("build.rs"), build_rs_content)?;
    
    // Create Cargo.toml
    let cargo_toml_content = r#"
[package]
name = "zeroproto-schema-example"
version = "0.1.0"
edition = "2021"

[dependencies]
zeroproto = { path = "../../crates/zeroproto" }

[build-dependencies]
zeroproto-compiler = { path = "../../crates/zeroproto-compiler" }
"#;
    
    fs::write(temp_dir.join("Cargo.toml"), cargo_toml_content)?;
    
    // Create src/main.rs
    let main_rs_content = r#"
mod generated;

use generated::user::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏗️  Building complex user with schema-generated types");
    
    // Create user settings
    let mut settings_builder = UserSettingsBuilder::new();
    settings_builder.set_theme(Theme::Dark);
    settings_builder.set_notifications_enabled(true);
    settings_builder.set_max_friends(500);
    let settings_data = settings_builder.finish();
    
    // Create user profile
    let mut profile_builder = UserProfileBuilder::new();
    profile_builder.set_bio("Rust developer and ZeroProto enthusiast");
    profile_builder.set_avatar_url("https://example.com/avatar.jpg");
    profile_builder.set_settings(&settings_data);
    let profile_data = profile_builder.finish();
    
    // Create the main user
    let mut user_builder = UserBuilder::new();
    user_builder.set_user_id(12345);
    user_builder.set_username("alice");
    user_builder.set_email("alice@example.com");
    user_builder.set_age(28);
    user_builder.set_status(Status::Active);
    
    // Add tags
    let tags = vec!["rust", "programming", "zeroproto", "performance"];
    user_builder.set_tags(&tags);
    
    // Set profile
    user_builder.set_profile(&profile_data);
    
    // Serialize the user
    let user_data = user_builder.finish();
    println!("✅ Serialized user: {} bytes", user_data.len());
    
    // Read the user (zero-copy!)
    let user = UserReader::from_slice(&user_data)?;
    
    println!("\n📖 User Information:");
    println!("   ID: {}", user.user_id()?);
    println!("   Username: {}", user.username()?);
    println!("   Email: {}", user.email()?);
    println!("   Age: {}", user.age()?);
    println!("   Status: {:?}", user.status()?);
    
    // Read tags
    let tags_reader = user.tags()?;
    println!("   Tags:");
    for tag in tags_reader.iter() {
        println!("     - {}", tag?);
    }
    
    // Read nested profile
    let profile = user.profile()?;
    println!("   Profile:");
    println!("     Bio: {}", profile.bio()?);
    println!("     Avatar: {}", profile.avatar_url()?);
    
    // Read nested settings
    let settings = profile.settings()?;
    println!("   Settings:");
    println!("     Theme: {:?}", settings.theme()?);
    println!("     Notifications: {}", settings.notifications_enabled()?);
    println!("     Max Friends: {}", settings.max_friends()?);
    
    // Demonstrate zero-copy benefits
    println!("\n🔍 Zero-Copy Analysis:");
    let username = user.username()?;
    println!("   Username references original buffer: {}", 
             username.as_ptr() as usize >= user_data.as_ptr() as usize &&
             username.as_ptr() as usize < user_data.as_ptr() as usize + user_data.len());
    
    println!("\n🎉 Schema-based compilation successful!");
    Ok(())
}
"#;
    
    fs::write(temp_dir.join("src/main.rs"), main_rs_content);
    
    println!("📁 Created example project in: {}", temp_dir.display());
    
    // Now compile the schema using our compiler
    println!("\n🔧 Compiling schema...");
    
    // Use the current zeroproto compiler
    std::env::set_current_dir(&temp_dir)?;
    
    // Compile the schema
    zeroproto_compiler::compile_multiple(
        &["schemas/user.zp"],
        "src/generated"
    )?;
    
    println!("✅ Schema compiled successfully!");
    
    // Show generated code
    let generated_path = temp_dir.join("src/generated/user.rs");
    if generated_path.exists() {
        let generated_code = fs::read_to_string(&generated_path)?;
        println!("\n📄 Generated code preview (first 500 chars):");
        println!("{}", &generated_code[..std::cmp::min(500, generated_code.len())]);
        if generated_code.len() > 500 {
            println!("... (truncated)");
        }
    }
    
    println!("\n🎯 Example completed successfully!");
    println!("   You can run the full example with:");
    println!("   cd {}", temp_dir.display());
    println!("   cargo run");
    
    Ok(())
}
