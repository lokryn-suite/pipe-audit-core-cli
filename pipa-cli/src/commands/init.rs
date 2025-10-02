use pipa::init::init_project;

/// Initialize a new project in the current directory.
///
/// Delegates to `pipa::init::init_project()`, which scaffolds the
/// necessary folder structure and starter files. Prints a success
/// message or an error if initialization fails.
///
/// Called from `main.rs` when the user runs:
/// ```bash
/// pipa init
/// ```
pub async fn init_project() {
    match init_project() {
        Ok(msg) => println!("{}", msg),
        Err(e) => eprintln!("❌ {}", e),
    }
}
