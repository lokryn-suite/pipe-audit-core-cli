<!-- .github/copilot-instructions.md - guidance for AI coding agents -->
# data-quality — AI agent instructions

This repository is a small Rust CLI for validating tabular data (CSV/Parquet) against TOML "contracts" (schemas + rules). Use these instructions to be productive quickly.

1. Big picture
   - Main CLI: `src/main.rs` wires `clap` to `commands::validate` and `commands::run`.
   - Validation flow: `runner::validate_file` loads a contract (`contracts::load_contract_for_file`), selects a driver (`drivers::get_driver`), loads a `polars::DataFrame`, and validates it with `engine::validate_dataframe` which invokes validators in `validators/*`.
   - Connectors vs drivers: Connectors (`src/connectors/*`) provide listing/fetching remote files (S3/GCS/Azure/SFTP/local). Drivers (`src/drivers/*`) read local files into DataFrames (CSV/Parquet). Use connectors to obtain local paths or streams; use drivers to load into `polars::DataFrame`.

2. Key files and examples (use these as canonical examples)
   - `src/runner.rs` — single-file validation entrypoint. Example: `validate_file("data/people.csv")`.
   - `src/contracts/schema.rs` — contract lookup: it maps `data/<name>.*` → `contracts/<name>.toml`.
   - `src/validators/*.rs` — concrete rule implementations. Example: `validators/column.rs::apply_column_contract` logs per-rule results using `logging::log_validation_event`.
   - `src/logging.rs` — JSONL audit logs live in `logs/audit-YYYY-MM-DD.jsonl`; logs are sealed (hashed) daily via `hash_ledger.txt`.
   - `src/connectors.rs` + `src/connectors/local.rs` — connector trait and local implementation; prefer `from_connection_string` factory for scheme selection.

3. Project-specific patterns and conventions
   - Contracts are TOML files under `contracts/` and must be named to match the data filename stem (see `load_contract_for_file`). If missing, code panics with a clear message.
   - Logging: use `logging::log_validation_event(...)` to record validation events. Events are emitted as JSON to daily audit files.
   - Error handling: many places use `unwrap`/`panic!` for missing contracts or unsupported file types — preserve behavior when modifying unless intentionally making error handling more robust, and call out the change in PR notes.
   - Drivers return `polars::prelude::DataFrame` via a `DataSource` trait. To add a new file type, implement `DataSource` in `src/drivers/<type>.rs` and update `get_driver`.
   - Connectors return `Box<dyn Read>` for fetched data. `LocalConnector::fetch` returns `File`. S3/GCS connectors parse URLs via `url::Url` (see `from_connection_string`).

4. Developer workflows / common commands
   - Build: `cargo build` or `cargo build --release`.
   - Quick check: `cargo check` (fast). Run tests (if added) with `cargo test`.
   - Run CLI locally: from repo root

     ```bash
     cargo run -- --help
     cargo run -- validate data/people.csv
     cargo run -- run --all
     ```

   - Linting/formatting: use `cargo fmt` and `cargo clippy` as needed.

5. Integration points & external dependencies
   - Polars (`polars`, `polars-io`) for DataFrame operations (see `Cargo.toml`). Keep transforms within Polars APIs where possible.
   - DuckDB, Postgres, S3/GCS/Azure crates appear in dependencies or connectors; connectors construct `url::Url` and use scheme-specific clients.
   - Audit logs: `logs/` must be present or `logging::init_logging` will create it; CI or automation relying on logs should read `logs/hash_ledger.txt` for sealed files.

6. Editing guidelines for AI agents
   - Preserve CLI behavior and `contracts/` naming convention unless changing `load_contract_for_file` intentionally — state migration steps in the PR.
   - When adding validators, follow `validators/column.rs` pattern: compute metric → call `logging::log_validation_event(...)` with `event="validation"` and `result` set to `pass|fail|skipped`.
   - Prefer non-breaking changes. If a change alters logging schema, update `src/logging.rs` and note backwards compatibility impact on `logs/` consumers.
   - Avoid introducing network calls in lint/format steps. Network clients should be isolated to connectors and optionally feature-gated.

7. Edge cases discovered
   - Contract loading panics if contract file is missing. Consider explicit errors for user-facing commands.
   - `drivers::get_driver` panics on unknown extensions — adding new drivers requires updating this function.
   - Many validators assume string columns for string rules; casting/skip semantics are used in `validators/column.rs` (e.g., Range skips non-numeric).

If anything is unclear or you want the instructions expanded with examples (e.g., how to add a new connector or validator), tell me what to add and I will iterate.
