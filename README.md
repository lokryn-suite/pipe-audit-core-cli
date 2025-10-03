
# Pipe Audit

**Universal data validation and compliance audit engine for structured data pipelines.**

`pipe audit` helps **data engineers, developers, and technical managers** enforce reproducible, compliance‑grade validation without heavy setup.  
Use it as a **Rust library** (`pipe-audit-core`), a **standalone CLI** (`pipe-audit` → installs the `pipa` binary), or both.

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
cargo install pipe-audit
```

This provides the `pipa` binary:

```bash
pipa init
pipa run --all
```

### Library
If you want to embed the engine in your own Rust project, add the core library:

```toml
[dependencies]
pipe-audit-core = "0.1"
```

---

## 🛠️ Quick Start (CLI)

Validate a contract:

```bash
pipa contract validate contracts/example.toml
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

👉 For full guides, examples, and contributor onboarding, see the [📚 Documentation](https://docs.lokryn.com).

---

## 📦 Library Usage

```rust
use pipe_audit_core::engine::run_contract_validation;

fn main() {
    run_contract_validation("contracts/example.toml").unwrap();
}
```

---

## 📄 License

- **`pipe-audit-core`** (the library) is licensed under the **Mozilla Public License 2.0 (MPL‑2.0)**.  
- **`pipe-audit`** (the CLI, installs as `pipa`) is licensed under the **GNU General Public License v3.0 or later (GPL‑3.0‑or‑later)**.  

See the [`LICENSES/`](./LICENSES) directory for full texts.

---

## 📚 Documentation & Community

- Full docs: [https://docs.lokryn.com](https://docs.lokryn.com)  
- Join the discussion on [Discord](https://discord.gg/4JJT9qEfCA)

---

## 🔮 Roadmap
- Richer docs with end‑to‑end examples  
- Expanded connectors (databases, streaming sources)  
- Containerized API service built on `pipe-audit-core`  
```

