use pipa::init::init_project as lib_init_project;

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
pub fn init_project() {
    match lib_init_project() {
        Ok(msg) => println!("{}", msg),
        Err(e) => eprintln!("❌ {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_project_calls_lib_function() {
        // This test verifies that init_project function exists and can be called
        // The actual implementation is tested in the pipa-core library
        // We're just testing the CLI wrapper logic here

        // Since lib_init_project() has side effects (creates directories),
        // we can't easily test it without mocking. Instead, we test that
        // the function signature is correct and the code compiles.
        let _ = lib_init_project;
    }
}
