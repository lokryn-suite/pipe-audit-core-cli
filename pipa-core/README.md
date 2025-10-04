
# pipe-audit-core

**The core engine for universal data validation and compliance auditing in Rust.**

`pipe-audit-core` provides the building blocks for defining and running validation contracts against structured data. It is designed to be embedded in other Rust applications to enforce reproducible, compliance-grade data validation.

---

## ✨ Features
- **Cloud storage integration**: Built-in support for Azure Blob, Google Cloud Storage, and Amazon S3.
- **Compliance-grade audit logging**: Generate tamper-resistant, JSON-structured logs for every action.
- **Simple TOML configuration**: Define contracts, profiles, and validation rules programmatically or from TOML files.
- **A clean, modular API**: Interact with specific parts of the engine (contracts, profiles, health checks) through a clear and stable API.

---

## 🚀 Installation

Add `pipe-audit-core` as a dependency in your project's `Cargo.toml`:


[dependencies]
pipe-audit-core = "0.1" # Replace with the latest version


-----

## 📦 API Overview

The library's functionality is exposed through several purpose-driven modules:

  * **`pipe_audit_core::contract`**: Manage and execute validation contracts.
  * **`pipe_audit_core::profile`**: List and test connection profiles.
  * **`pipe_audit_core::health`**: Run system-level health and connectivity checks.
  * **`pipe_audit_core::logs`**: Verify the integrity of audit logs.
  * **`pipe_audit_core::init`**: Initialize a new project structure with example files.
  * **`pipe_audit_core::run`**: A simple, top-level way to run a contract against a file. 
  * **`pipe_audit_core::prelude`**: A module that re-exports the most commonly used types for convenience.

👉 For the full API reference, generate the documentation locally by running `cargo doc --open`.

-----

## 📄 License

This library (`pipe-audit-core`) is licensed under the **Mozilla Public License 2.0 (MPL-2.0)**. See the `LICENSES/` directory in the root of the project for the full text.

-----

## 📚 Documentation & Community

  - Full docs: [https://docs.lokryn.com](https://docs.lokryn.com)
  - Join the discussion on [Discord](https://discord.gg/4JJT9qEfCA)
