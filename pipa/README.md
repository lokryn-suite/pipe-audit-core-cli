
# pipe-audit (pipa CLI)

**A standalone CLI for universal data validation and compliance audits.**

The `pipe-audit` crate provides the `pipa` binary, a command-line tool that helps **data engineers, developers, and technical managers** enforce reproducible, compliance-grade validation without heavy setup. It is built on the `pipe-audit-core` engine.

---

## ✨ Features
- **Cloud storage integration**: Azure Blob, Google Cloud Storage, Amazon S3
- **Compliance-grade audit logging**: Tamper-resistant, JSON-structured logs for every action
- **Simple TOML configuration**: Define contracts, profiles, and validation rules in plain TOML
- **Standalone and easy to install**: No external dependencies besides the Rust toolchain

---

You should add this information directly to the "Installation" section of your `pipe-audit` CLI's `README.md` file.

It's best to present the installers as the primary, recommended method, and then add a subsection for advanced users who want to build from source with `cargo install`, clearly listing the necessary requirements.

-----


## 🚀 Installation

### Recommended: Installers for Windows, macOS, and Linux

The easiest way to install the `pipa` CLI is to download a pre-compiled installer for your operating system from our latest GitHub Release.

**[➡️ Download the Latest Release from GitHub](https://github.com/lokryn-suite/pipe-audit-core-cli/releases/latest)**

* **Windows:** Download the `.msi` file and run the graphical installer.
* **macOS:** Install via Homebrew (recommended) `brew install lokryn-suite/pipa/pipe-audit`
* **Linux:** Use the universal shell script.

---

### Alternate Method: Build from Source with `cargo`

This method is for advanced users who have a Rust development environment set up.

**1. Install the CLI:**
```bash
cargo install pipe-audit
````

**2. Windows Requirements:**
If you are on Windows, `cargo install` needs to compile the code from source, which requires the **Visual Studio C++ Build Tools**. You can download them directly from Microsoft.

  * **[Download Microsoft C++ Build Tools](https://www.google.com/search?q=https://visualstudio.microsoft.com/visual-tools/build-tools/)**

During the installation, make sure you select the "C++ build tools" workload.

-----


## 🛠️ Quick Start

First, initialize a new project structure with example contracts and configurations:

```bash
pipa init
```

Validate a specific contract:

```bash
pipa contract validate contracts/example.toml
```

Run all validation contracts defined in your project:

```bash
pipa run --all
```

Check system health and connectivity:

```bash
pipa health
```

List available validation profiles:

```bash
pipa profile list
```

Verify the integrity of an audit log:

```bash
pipa logs verify ./examples/logs/test.log
```

👉 For full guides and examples, see the [📚 Documentation](https://docs.lokryn.com).

-----

## 📄 License

This tool (`pipe-audit`, which installs as `pipa`) is licensed under the **GNU General Public License v3.0 or later (GPL-3.0-or-later)**. See the `LICENSES/` directory in the root of the project for the full text.

-----

## 📚 Documentation & Community

  - Full docs: [https://docs.lokryn.com](https://docs.lokryn.com)
  - Join the discussion on [Discord](https://discord.gg/4JJT9qEfCA)





