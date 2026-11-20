# Strict Rust Code Rigor

Achieving maximum strictness in the Rust configuration you provided requires combining native `rustc` lints, `clippy` lints, and specific coding patterns. By enabling this configuration, you are essentially forcing all code to be memory-safe, panic-free, fully documented, and robustly typed.

Below is the definitive guide on how to configure your project and write code that satisfies these demands.

---

## 1. The Global Lint Configuration

Instead of adding `#![deny(...)]` to every file, modern Rust allows you to define workspace-wide lints in your `Cargo.toml`. This ensures the rules cannot be bypassed.

Add this block to your `Cargo.toml` (or `[workspace.lints]` in a monorepo):

```toml
[lints.rust]
unsafe_code = "deny"
missing_docs = "deny"
missing_debug_implementations = "deny"
unreachable_pub = "deny"
unused_results = "deny"
unused_qualifications = "deny"
trivial_casts = "deny"
trivial_numeric_casts = "deny"
unused_extern_crates = "deny"

[lints.clippy]
# Level 1: The Main Groups
all = { level = "deny", priority = -1 }
pedantic = { level = "deny", priority = -1 }
nursery = { level = "warn", priority = -1 }

# Level 2: The "Give Me a Reason" Rule
allow_attributes = "deny"
allow_attributes_without_reason = "deny"

# Level 3: Panic Prevention & Tech Debt
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
unimplemented = "deny"
unreachable = "deny"
dbg_macro = "deny"
print_stdout = "warn"

# Dependency Management
wildcard_dependencies = "deny"
cargo = { level = "warn", priority = -1 }
multiple_crate_versions = "deny"
```

---

## 2. Writing Panic-Free Code (`unwrap`, `expect`, `panic`)

Because `unwrap()`, `expect()`, and `panic!()` are forbidden, **all** failure states must be propagated as data using `Result` or handled safely. 

### ❌ The Old Way (Banned)
```rust
fn load_config() -> Config {
    let file = std::fs::read_to_string("config.toml").unwrap(); // ❌ panics
    let config: Config = toml::from_str(&file).expect("Invalid toml"); // ❌ panics
    config
}
```

### ✅ The Strict Way (Propagating Errors)
Use a dedicated error type (like `thiserror` for libraries or `anyhow` for applications) and the `?` operator.

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),
}

/// Loads the application configuration.
pub fn load_config() -> Result<Config, ConfigError> {
    let file = std::fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&file)?;
    
    Ok(config)
}
```

### ✅ The Strict Way (Handling Options with `let else`)
When dealing with `Option<T>` without panicking, use the `let else` syntax to return early or propagate custom errors.

```rust
/// Extracts the port from the host string.
pub fn extract_port(host: &str) -> Result<u16, ConfigError> {
    let Some(port_str) = host.split(':').last() else {
        return Err(ConfigError::MissingPort);
    };
    
    // Instead of parsing with unwrap, we map the error
    port_str.parse::<u16>().map_err(|_| ConfigError::InvalidPort)
}
```

---

## 3. Strict Visibility (`unreachable_pub`)

Rust defaults to `pub` meaning "public to the world." With `unreachable_pub = "deny"`, you cannot make something `pub` if it isn't actually exported from the crate root. 

If a struct is only used inside your crate, use `pub(crate)`.

```rust
// ❌ Denied: This is public, but not exported from the library.
// pub struct InternalWorker;

// ✅ Allowed: Explicitly scoped to the crate.
pub(crate) struct InternalWorker;

// ✅ Allowed: Scoped to the parent module.
pub(super) struct ModuleHelper;
```

---

## 4. Documentation & Debug Derives

Every single public item (structs, enums, traits, functions) must have a doc comment (`///`) and every public struct/enum must derive `Debug`.

```rust
// ❌ Denied: Missing docs and missing #[derive(Debug)]
// pub struct User {
//     id: u64,
// }

/// Represents a registered user in the system.
#[derive(Debug, Clone)]
pub struct User {
    /// The unique identifier for the user.
    pub id: u64,
}
```

---

## 5. Justifying Exceptions (`allow_attributes_without_reason`)

There will be times you *must* break a rule. Because you denied `allow_attributes`, you must use `#[expect(...)]` instead of `#[allow(...)]`. `#[expect]` acts like an assertion: if the code inside stops triggering the lint, the compiler will warn you to remove the attribute.

Furthermore, `allow_attributes_without_reason` forces you to explain *why*.

```rust
/// Computes the hash of the data.
pub fn compute_hash(data: &[u8]) -> u64 {
    // ❌ Denied: No reason provided, and uses `allow`
    // #[allow(clippy::cast_possible_truncation)]
    // let shortened = data.len() as u32;

    // ✅ Allowed: Uses `expect` and provides a mandatory reason
    #[expect(
        clippy::cast_possible_truncation, 
        reason = "Data length is validated upstream to never exceed u32::MAX"
    )]
    let shortened = data.len() as u32;
    
    // ...
    0
}
```

---

## 6. Avoiding "Tech Debt" Macros (`todo`, `unimplemented`, `unreachable`)

Placeholder macros cause runtime panics. Under your configuration, they are treated as errors.

*   **Don't leave empty functions:** If a function isn't ready, don't write it. If it must exist to satisfy a trait, it must return a valid error type.
*   **Exhaustive Matching instead of `unreachable!()`:** Rely on the type system to prove a branch is unreachable, rather than using the `unreachable!()` macro.

```rust
pub enum State {
    Running,
    Stopped,
}

pub fn handle_state(state: State) {
    match state {
        State::Running => { /* ... */ },
        State::Stopped => { /* ... */ },
        // ❌ Denied: Do not use unreachable! or wildcard matches for known enums
        // _ => unreachable!("Added new states?"),
    }
}
```

---

## 7. No Console Printing (`print_stdout`, `dbg_macro`)

Do not use `println!` or `dbg!`. They tie your application to standard output and lack log levels. Instead, use the `tracing` ecosystem.

```rust
use tracing::{info, warn, error};

/// Processes a background job.
pub fn process_job(id: u64) {
    // ❌ Denied
    // println!("Processing job {}", id);
    // dbg!(id);

    // ✅ Allowed
    info!(job_id = id, "Processing job");
    
    if id == 0 {
        warn!("Received invalid job ID");
    }
}
```

---

## 8. Handling Unused Results (`unused_results`)

If a function returns a value (like a `Result`), you cannot implicitly ignore it. You must either handle the error, propagate it, or explicitly discard it using `let _ =`.

```rust
use std::fs::File;
use std::io::Write;

/// Logs data to a file on a best-effort basis.
pub fn best_effort_log(data: &str) {
    let mut file = match File::create("log.txt") {
        Ok(f) => f,
        Err(_) => return, // Handle gracefully
    };

    // ❌ Denied: `write_all` returns a Result that is being ignored.
    // file.write_all(data.as_bytes());

    // ✅ Allowed: Explicitly acknowledge that we don't care if this fails.
    let _ = file.write_all(data.as_bytes());
}
```

---

## 9. Dependency Rigor (`wildcard_dependencies`, `multiple_crate_versions`)

Your configuration forbids wildcard dependencies (`*`) and duplicate dependencies in your lockfile.

**In your `Cargo.toml`:**
```toml
[dependencies]
# ❌ Denied
# serde = "*"
# anyhow = "^1.0" (While allowed by Cargo, strict teams prefer exactness)

# ✅ Allowed
serde = "1.0.197"
```

**Resolving `multiple_crate_versions`:**
If `clippy` complains that you have multiple versions of `syn` or `regex` in your `Cargo.lock`, you can resolve them by running:
```bash
cargo update
```
If transitive dependencies strictly require incompatible versions, you can add an exception in a `clippy.toml` file at the root of your project:
```toml
# clippy.toml
allowed-duplicate-crates = ["syn", "regex"]
```

---

## 10. Architecting for Strict Rust Compliance (Avoiding "Brute Force")

Do not simply write a massive block of code and "brute force" the compiler loop until it passes. You must proactively architect your code to inherently comply with strict lints (`pedantic`, `unwrap_used`, `nursery`) and borrow-checker constraints from the start.

### The Methodology for Strict Compliance:
1. **Lifetime & Borrowing Elegance:** 
   Instead of fighting the borrow checker with pervasive explicit lifetimes (`'a`), design your domain structs with **owned types** (`String`, `Vec<u8>`) when crossing async or thread boundaries. If borrowing is mathematically required for performance, use `std::borrow::Cow` or encapsulate shared state cleanly via `Arc<T>`.
2. **Type Signature & Trait Boundaries:**
   Use Interface-Driven Architecture. Define clear, isolated traits (e.g., `EmailProvider`, `ObjectStorage`) using `async-trait` or native async traits. Bound your implementations cleanly. Avoid massive generic bounds (`<T: TraitA + TraitB + TraitC>`) in domain logic; use concrete structs that implement the traits.
3. **Exhaustive Error Modeling:**
   Never use `unwrap()` or `expect()`. Model your domain errors precisely using `thiserror`. When bubbling up errors to handlers or IPC layers, convert them cleanly into `anyhow::Error` or map them to user-facing API errors. 

### The Micro-Verification Loop & Atomic Commits
The AST Architect succeeded because it didn't write the whole codebase at once. It followed this pattern:
1. **Targeted Implementation:** Write one logical isolated component (e.g., the Domain Models).
2. **Micro-Verification:** Run `cargo check --workspace` immediately to resolve isolated type signature and lifetime errors before they cascade into other modules.
3. **Atomic Saves (Telling a Story):** Once that *single component* compiles cleanly (exit code 0), run `git status`. Do **NOT** use `git add .`. Execute `git add <file>` and make a distinct, atomic conventional commit (e.g., `feat(models): implement core domain structures`).
4. **Repeat:** Move to the next component (e.g., Traits), verify, and commit. 

By grouping your changes into distinct, logically isolated atomic commits, you ensure your PR "tells a story" and keeps compiler feedback loops tiny and manageable.
