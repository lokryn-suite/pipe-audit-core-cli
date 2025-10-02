/// The `commands` module contains the async implementations
/// of each CLI subcommand defined in `cli.rs`.
///
/// Each submodule corresponds to a top-level CLI command
/// (or a nested command group) and provides the actual
/// execution logic that `main.rs` dispatches into.
///
/// Structure:
/// - `cli.rs` → defines the CLI grammar (flags, subcommands)
/// - `main.rs` → parses args and dispatches to `commands::*`
/// - `commands/*` → contains the async functions that call into
///                  the engine (`pipa::*`) to do real work.

/// Contract management commands.
/// Implements `list`, `validate`, and `show` for contracts.
pub mod contract;

/// System health check command.
/// Implements `commands::health::run()`, which verifies
/// environment setup and connector readiness.
pub mod health;

/// Project initialization command.
/// Implements `commands::init::init_project()`, which scaffolds
/// a new project directory with starter files.
pub mod init;

/// Log management commands.
/// Implements `commands::logs::verify()`, which checks
/// integrity of sealed logs or logs for a given date.
pub mod logs;

/// Profile management commands.
/// Implements `commands::profile::{list, test}`, which
/// enumerate profiles and test connectivity.
pub mod profile;

/// Run commands.
/// Implements `commands::run::{run_all, run_single}`, which
/// execute contract validation workflows.
pub mod run;
