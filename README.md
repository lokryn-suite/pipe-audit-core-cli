
# Pipe Audit (pipa)

**Universal data validation and compliance audit engine for structured data pipelines.**

`pipa` helps **data engineers, developers, and technical managers** enforce reproducible, compliance‑grade validation without heavy setup.  
Use it as a **Rust library** (`pipa-core`), a **standalone CLI** (`pipa`), or both.

---

## ✨ Features
- **Cloud storage integration**: Azure Blob, Google Cloud Storage, Amazon S3  
- **Compliance‑grade audit logging**: tamper‑resistant, JSON‑structured logs for every action  
- **Simple TOML configuration**: define contracts, profiles, and validation rules in plain TOML  
- **Flexible usage**: embed as a Rust library or run as a CLI tool  

---

## 🚀 Installation

### CLI
Install the CLI globally (requires a recent Rust toolchain, edition 2024):

```bash
cargo install pipa
```

This provides the `pipa` binary.

### Library
If you want to embed the engine in your own Rust project, add the core library:

```toml
[dependencies]
pipa-core = "0.1"
```

---

## 🛠️ Quick Start (CLI)

Initiate a project:

```bash
pipa init
```

Validate a contract:

```bash
pipa contract validate contracts/people.toml
```

Check system health:

```bash
pipa health
```

List available profiles:

```bash
pipa profile list
```

Verify logs:

```bash
pipa logs verify ./examples/logs/test.log
```

👉 More commands and advanced usage will be documented in the upcoming docs.

---

## 📦 Library Usage

```rust
use pipa_core::engine::run_contract_validation;

fn main() {
    run_contract_validation("contracts/people.toml").unwrap();
}
```

---

## 📄 License

- **`pipa-core`** (the library) is licensed under the **Mozilla Public License 2.0 (MPL‑2.0)**.  
- **`pipa`** (the CLI) is licensed under the **GNU General Public License v3.0 or later (GPL‑3.0‑or‑later)**.  

See the [`LICENSES/`](./LICENSES) directory for full texts.

---

## 🔮 Roadmap
- Richer docs with end‑to‑end examples  
- Expanded connectors (databases, streaming sources)  
- Containerized API service built on `pipa-core`  

---

