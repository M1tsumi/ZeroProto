# ZeroProto Tutorial

Welcome! This tutorial will take you from zero to hero with ZeroProto. By the end, you'll understand how zero-copy serialization works and how to use it effectively in your projects.

## What We'll Cover

1. [Getting Started](#getting-started) - Installation and first steps
2. [The Big Idea](#the-big-idea) - Why zero-copy matters
3. [Writing Schemas](#writing-schemas) - Defining your data structures
4. [Code Generation](#code-generation) - Turning schemas into Rust code
5. [Building and Reading Messages](#building-and-reading-messages) - The fun part
6. [Going Deeper](#going-deeper) - Advanced patterns
7. [Making It Fast](#making-it-fast) - Performance tips
8. [When Things Go Wrong](#when-things-go-wrong) - Error handling
9. [Tips and Tricks](#tips-and-tricks) - Best practices

## Getting Started

Let's get you up and running.

### Installation

Add these to your `Cargo.toml`:

```toml
[dependencies]
zeroproto = "0.3.0"

[build-dependencies]
zeroproto-compiler = "0.3.0"
```

### The Quick Way

If you want to skip the setup, use our CLI:

```bash
zeroproto init my-project
cd my-project
cargo build
```

Boom! You've got a working project with:
- `Cargo.toml` - Dependencies already configured
- `build.rs` - Schema compilation set up
- `schemas/` - A place for your `.zp` files
- `src/main.rs` - A working example to play with

Need to manage a giant schema tree? Every CLI command (`compile`, `watch`, `check`, `inspect`) now accepts the same filtering flags:

```bash
# Only compile tenant A
zeroproto compile schemas/ --include "tenants/a/**/*.zp"

# Skip a legacy folder everywhere
zeroproto watch schemas/ --exclude "**/legacy/**"

# Print stats instead of generating code
zeroproto inspect schemas/ --verbose
```

Includes/excludes are glob patterns resolved relative to the input path, and the CLI always prints which files were included or skipped so you can double-check coverage.

## The Big Idea

### What Makes Zero-Copy Special?

Most serialization libraries work like this:

```rust
// Traditional approach: allocate and copy
let user: User = serde_json::from_str(&json_string)?;
// ^ This allocates memory for every string, every vector, everything
```

ZeroProto works differently:

```rust
// ZeroProto: just read from the buffer
let user = UserReader::from_slice(&buffer)?;
let name: &str = user.name()?;
// ^ This is a slice into the original buffer. No allocation!
```

The data stays where it is. We just read it in place. This is why ZeroProto is so fast.

### How It Works

You write a schema:

```zp
message User {
    user_id: u64;
    name: string;
    email: string;
}
```

The compiler generates Rust code:

```rust
// You get a reader (for zero-copy deserialization)
pub struct UserReader<'a> { /* ... */ }

impl<'a> UserReader<'a> {
    pub fn user_id(&self) -> Result<u64> { /* ... */ }
    pub fn name(&self) -> Result<&'a str> { /* ... */ }  // Note the lifetime!
    pub fn email(&self) -> Result<&'a str> { /* ... */ }
}

// And a builder (for serialization)
pub struct UserBuilder { /* ... */ }
```

Notice the `'a` lifetime on the reader? That's the magic. The strings you get back are borrowed from the original buffer.

## Writing Schemas

Schemas are where you define your data structures. They live in `.zp` files.

### The Basics

Here are all the types you can use:

```zp
message AllTheTypes {
    // Integers (signed and unsigned)
    tiny: i8;
    small: i16;
    medium: i32;
    big: i64;
    tiny_unsigned: u8;
    small_unsigned: u16;
    medium_unsigned: u32;
    big_unsigned: u64;
    
    // Floating point
    float_val: f32;
    double_val: f64;
    
    // Other primitives
    flag: bool;
    name: string;   // UTF-8 string
    raw_data: bytes; // Raw byte array
}
```

### Enums

Enums need explicit values (we don't auto-assign them):

```zp
enum Status {
    Inactive = 0;
    Active = 1;
    Pending = 2;
    Banned = 3;
}

message User {
    user_id: u64;
    status: Status;
}
```

### Nesting Messages

Messages can contain other messages:

```zp
message Address {
    street: string;
    city: string;
    country: string;
    zip: string;
}

message User {
    user_id: u64;
    name: string;
    home_address: Address;  // Nested!
    work_address: Address;  // You can have multiple
}
```

### Vectors (Lists)

Use square brackets for collections:

```zp
message User {
    user_id: u64;
    tags: [string];      // List of strings
    scores: [u32];       // List of numbers
    friend_ids: [u64];   // List of IDs
    addresses: [Address]; // List of messages!
}
```

## Code Generation

The compiler turns your schemas into Rust code. Here's how to set it up.

### The build.rs File

Create `build.rs` in your project root:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This compiles everything in the schemas/ directory
    zeroproto_compiler::build()?;
    Ok(())
}
```

That's it! Cargo will run this before building your crate.

### Want More Control?

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile specific files to a specific location
    zeroproto_compiler::compile_multiple(
        &["schemas/user.zp", "schemas/post.zp"],
        "src/generated"
    )?;
    Ok(())
}
```

### Project Layout

Here's what a typical project looks like:

```
my-project/
├── Cargo.toml
├── build.rs           # Runs the compiler
├── schemas/
│   ├── user.zp        # Your schema files
│   └── post.zp
└── src/
    ├── main.rs
    └── generated/     # Generated code goes here
        ├── mod.rs
        ├── user.rs
        └── post.rs
```

## Building and Reading Messages

This is where ZeroProto shines. Let's see it in action.

### Creating a Message

```rust
use generated::user::*;

fn create_user() -> Vec<u8> {
    let mut builder = UserBuilder::new();
    
    // Set simple fields
    builder.set_user_id(12345);
    builder.set_name("Alice");
    builder.set_email("alice@example.com");
    
    // Nested messages: build them separately, then attach
    let mut addr = AddressBuilder::new();
    addr.set_street("123 Main St");
    addr.set_city("Portland");
    addr.set_country("USA");
    addr.set_zip("97201");
    let addr_data = addr.finish();
    
    builder.set_home_address(&addr_data);
    
    // Vectors are easy
    builder.set_tags(&["developer", "rust-lover", "coffee-addict"]);
    builder.set_friend_ids(&[1001, 1002, 1003]);
    
    builder.finish()  // Returns Vec<u8>
}
```

### Reading a Message (Zero-Copy!)

```rust
fn read_user(data: &[u8]) -> Result<(), zeroproto::Error> {
    // This doesn't copy anything - it just sets up pointers into `data`
    let user = UserReader::from_slice(data)?;
    
    // Primitives are copied (they're small)
    let id: u64 = user.user_id()?;
    
    // Strings are borrowed - no allocation!
    let name: &str = user.name()?;
    let email: &str = user.email()?;
    
    println!("User {}: {} ({})", id, name, email);
    
    // Nested messages work the same way
    let addr = user.home_address()?;
    println!("Lives in: {}", addr.city()?);
    
    // Iterate over vectors
    for tag in user.tags()?.iter() {
        println!("Tag: {}", tag?);
    }
    
    Ok(())
}
```

### Handling Errors

Every field access returns a `Result`. Here's how to handle them:

```rust
use zeroproto::Error;

fn safe_read(data: &[u8]) -> Result<String, Error> {
    let user = UserReader::from_slice(data)?;
    
    // Use ? for simple propagation
    let name = user.name()?;
    let email = user.email()?;
    
    Ok(format!("{} <{}>", name, email))
}

// Or handle specific errors
fn detailed_read(data: &[u8]) {
    match UserReader::from_slice(data) {
        Ok(user) => {
            match user.name() {
                Ok(name) => println!("Name: {}", name),
                Err(Error::InvalidOffset) => println!("Name field is corrupted"),
                Err(e) => println!("Error reading name: {:?}", e),
            }
        }
        Err(e) => println!("Invalid message: {:?}", e),
    }
}
```

## Going Deeper

### Validation

ZeroProto doesn't validate your business logic - that's your job:

```rust
fn validate_user(user: &UserReader) -> Result<(), String> {
    let id = user.user_id().map_err(|e| format!("Can't read ID: {:?}", e))?;
    if id == 0 {
        return Err("User ID can't be zero".into());
    }
    
    let email = user.email().map_err(|e| format!("Can't read email: {:?}", e))?;
    if !email.contains('@') {
        return Err("That's not a valid email".into());
    }
    
    Ok(())
}
```

### Working with Optional Data

In v0.3.0, we added optional fields:

```zp
message User {
    user_id: u64;
    name: string;
    nickname: string?;  // Optional!
    avatar_url: string?;
}
```

```rust
let user = UserReader::from_slice(&data)?;

// Optional fields return Result<Option<T>>
if let Some(nickname) = user.nickname()? {
    println!("Nickname: {}", nickname);
}

// Or check first
if user.has_nickname()? {
    println!("Nickname set!");
}
```

### Default Values

Also new in v0.3.0:

```zp
message Config {
    max_retries: u32 = 3;
    timeout_ms: u64 = 5000;
    debug: bool = false;
}
```

If a field isn't set, you get the default instead of an error.

## Making It Fast

ZeroProto is already fast, but here's how to squeeze out even more performance.

### Avoid Unnecessary Reads

```rust
// Bad: reads the same field twice
if user.name()?.len() > 0 {
    println!("Name: {}", user.name()?);
}

// Good: read once, use the result
let name = user.name()?;
if !name.is_empty() {
    println!("Name: {}", name);
}
```

### Batch Processing

If you're processing lots of messages, consider batching:

```rust
fn process_batch(messages: &[Vec<u8>]) -> Result<Vec<String>, zeroproto::Error> {
    let mut results = Vec::with_capacity(messages.len());
    
    for msg in messages {
        let user = UserReader::from_slice(msg)?;
        results.push(user.name()?.to_string());
    }
    
    Ok(results)
}
```

### Reuse Builders

Builders allocate memory. If you're creating lots of messages, reuse them:

```rust
let mut builder = UserBuilder::new();

for user_data in users {
    builder.clear();  // Reset for reuse
    builder.set_user_id(user_data.id);
    builder.set_name(&user_data.name);
    // ...
    let bytes = builder.finish();
    send_message(&bytes);
}
```

### Vector Iteration

Vector iteration is zero-copy too:

```rust
let tags = user.tags()?;

// This doesn't allocate - each tag is a slice into the original buffer
for tag in tags.iter() {
    process_tag(tag?);
}
```

## When Things Go Wrong

ZeroProto has a few error types you'll encounter:

### Error Types

```rust
use zeroproto::Error;

match err {
    Error::InvalidFieldType => {
        // The field exists but has the wrong type
        // Usually means corrupted data or schema mismatch
    }
    Error::InvalidOffset => {
        // Tried to read past the end of the buffer
        // Data is truncated or corrupted
    }
    Error::InvalidLength => {
        // A string or vector has an impossible length
        // Definitely corrupted data
    }
    Error::Io(io_err) => {
        // Underlying I/O error (rare)
    }
}
```

### Common Mistakes

**Using reserved field names:**
```zp
// Bad - 'type' is reserved
message Event {
    type: string;  // Compiler error!
}

// Good
message Event {
    event_type: string;
}
```

**Forgetting to handle Results:**
```rust
// Bad - will panic on error
let name = user.name().unwrap();

// Good
let name = user.name()?;
```

## Tips and Tricks

### Schema Design

- **Always give enums explicit values** - Makes schema evolution easier
- **Avoid reserved names** - `id`, `type`, `data`, `buffer` will cause problems
- **Keep messages focused** - One message per concept
- **Use vectors, not repeated fields** - They're more efficient

### Performance

- **Reuse builders** when creating many messages
- **Batch reads** when processing many messages
- **Don't copy strings** unless you need to keep them after the buffer is gone
- **Benchmark your actual use case** - synthetic benchmarks can be misleading

### Error Handling

- **Never `.unwrap()` in production** - Use `?` or proper error handling
- **Add context to errors** - "Failed to read user" is better than just the raw error
- **Validate early** - Check data as soon as you receive it

### Testing

- **Roundtrip test everything** - Serialize, deserialize, compare
- **Test edge cases** - Empty strings, empty vectors, max values
- **Test with corrupted data** - Make sure you get errors, not panics
- **Benchmark with realistic data** - Not just tiny test messages

## Putting It All Together

Here's a complete, working example:

**schemas/user.zp**
```zp
enum Status {
    Inactive = 0;
    Active = 1;
    Pending = 2;
}

message User {
    user_id: u64;
    username: string;
    email: string;
    status: Status;
    tags: [string];
}
```

**build.rs**
```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    zeroproto_compiler::build()?;
    Ok(())
}
```

**src/main.rs**
```rust
mod generated;
use generated::user::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a user
    let data = create_user();
    println!("Serialized {} bytes", data.len());
    
    // Read it back (zero-copy!)
    let user = UserReader::from_slice(&data)?;
    print_user(&user)?;
    
    // Validate it
    validate(&user)?;
    
    Ok(())
}

fn create_user() -> Vec<u8> {
    let mut builder = UserBuilder::new();
    builder.set_user_id(12345);
    builder.set_name("Alice");
    builder.set_email("alice@example.com");

    // Optionals get ergonomic setters
    builder.set_optional_nickname(Some("ally"));
    builder.clear_nickname(); // removes the field
    
    let mut friends = vec![1001, 1002, 1003];
    builder.set_friends(&friends);

    builder.set_tags(&["rust", "zeroproto", "speed"]);
    builder.finish()
}

fn print_user(user: &UserReader) -> Result<(), zeroproto::Error> {
    println!("User #{}", user.user_id()?);
    println!("  Name: {}", user.name()?);
    println!("  Email: {}", user.email()?);
    println!("  Status: {:?}", user.status()?);
    
    print!("  Tags: ");
    for (i, tag) in user.tags()?.iter().enumerate() {
        if i > 0 { print!(", "); }
        print!("{}", tag?);
    }
    println!();
    
    Ok(())
}

fn validate(user: &UserReader) -> Result<(), String> {
    if user.user_id().map_err(|e| e.to_string())? == 0 {
        return Err("User ID can't be zero".into());
    }
    
    let email = user.email().map_err(|e| e.to_string())?;
    if !email.contains('@') {
        return Err("Invalid email".into());
    }
    
    println!("Validation passed!");
    Ok(())
}
```

---

That's it! You now know everything you need to use ZeroProto effectively. The key things to remember:

1. **Zero-copy means fast** - Data stays in the buffer, we just read it
2. **Schemas define your types** - Write them in `.zp` files
3. **Builders create, Readers read** - Simple API
4. **Everything returns Result** - Handle your errors

Happy serializing!
