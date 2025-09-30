

# pipa

**Universal data validation and compliance audit engine for structured data pipelines.**

Designed for **data engineers, software developers, and technical managers** who need reproducible, compliance‑grade validation without heavy setup. Use it as a Rust library, a standalone CLI, or both.

---

## ✨ Features
- Cloud storage integration: Azure Blob, Google Cloud Storage, Amazon S3  
- Compliance‑grade audit logging: tamper‑resistant, JSON‑structured logs for every action  
- Simple TOML configuration: define contracts, profiles, and validation rules in plain TOML  
- Flexible usage: embed as a Rust library or run as a CLI tool  

---

## 🚀 Installation
You’ll need a recent Rust toolchain (edition 2024). Then run:

    cargo install pipa

This installs the `pipa` CLI globally.

---

## 🛠️ Quick Start

Validate a contract:

    pipa contract validate contracts/people.toml

Check system health:

    pipa health

List available profiles:

    pipa profile list

Verify logs:

    pipa logs verify ./examples/logs/test.log

👉 More commands and advanced usage will be documented in the upcoming docs.

---

## 📦 Library Usage

Add to your Cargo.toml:

    [dependencies]
    pipa = "0.1"

Then in Rust:

    use pipa::engine::run_contract_validation;

    fn main() {
        run_contract_validation("contracts/people.toml").unwrap();
    }

---

## 📄 License
This project is licensed under the **Lean Left License (attribution required)**.  
⚠️ Please confirm the exact license text (e.g. Elastic License 2.0, BUSL, or AGPL) and include it in a LICENSE file.

---

## 🔮 Roadmap
- Richer docs with end‑to‑end examples  
- Expanded connectors (databases, streaming sources)  

---
