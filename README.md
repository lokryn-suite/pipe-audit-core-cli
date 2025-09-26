# PipeAudit Core

Universal data validation for modern data engineering. Built on principles of compliance, security, and portability.

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/your-org/pipeaudit-core)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)](https://rustlang.org)

## Overview

PipeAudit provides boring, reliable data validation that works everywhere. It validates data against TOML-defined contracts, logs all results for compliance auditing, and integrates seamlessly into any data pipeline.

### Key Features

- **🔒 Compliance-First**: Cryptographically signed audit logs with tamper detection
- **🌐 Universal Compatibility**: Works with S3, Azure, GCS, local files, and SFTP
- **📝 Contract-Based**: Human-readable TOML contracts define validation rules
- **🔗 Pipeline Integration**: CLI, library, and API interfaces for any workflow
- **⚡ Performance**: Built in Rust for speed and reliability
- **🛡️ Security**: No sensitive data in logs, minimal attack surface

## Quick Start

### Installation

```bash
# From source
git clone https://github.com/your-org/pipeaudit-core
cd pipeaudit-core
cargo build --release

# Add to PATH
export PATH=$PATH:$(pwd)/target/release
```

### Basic Usage

1. **Create a contract** (`contracts/people.toml`):
```toml
[contract]
name = "people"
version = "0.1.0"
tags = ["pii", "critical"]

[[columns]]
name = "person_id"
validation = [
  { rule = "not_null" },
  { rule = "unique" },
  { rule = "pattern", pattern = "^[0-9a-fA-F-]{36}$" }
]

[[columns]]
name = "age"
validation = [
  { rule = "range", min = 0, max = 120 }
]

[source]
type = "local"
location = "data/people.csv"
```

2. **Run validation**:
```bash
pipa run people
```

3. **Check results**:
```bash
# All validation details are in structured logs
tail -f logs/audit-$(date +%Y-%m-%d).jsonl
```

## CLI Reference

### Data Validation
```bash
# Validate single contract
pipa run people

# Validate all contracts
pipa run --all
```

### Contract Management
```bash
# List available contracts
pipa contract list

# Validate contract syntax
pipa contract validate people.toml

# Show contract details
pipa contract show people
```

### Profile Management
```bash
# List configured profiles
pipa profile list

# Test profile connectivity
pipa profile test minio_raw
```

### Operations
```bash
# System health check
pipa health

# Verify log integrity
pipa logs verify
pipa logs verify --date 2025-01-24
```

## Configuration

### Profiles

Configure data sources in `profiles.toml`:

```toml
[minio_raw]
provider = "s3"
endpoint = "http://localhost:9000"
region = "us-east-1"
access_key = "${MINIO_ACCESS_KEY}"
secret_key = "${MINIO_SECRET_KEY}"
path_style = true
use_ssl = false

[prod_s3]
provider = "s3"
region = "us-west-2"
access_key = "${AWS_ACCESS_KEY}"
secret_key = "${AWS_SECRET_KEY}"

[azure_storage]
provider = "azure"
account = "${AZURE_STORAGE_ACCOUNT}"
key = "${AZURE_STORAGE_KEY}"
```

### Environment Variables

```bash
# Executor identity (for audit logs)
export PIPEAUDIT_EXECUTOR_ID="airflow-prod"

# Logging level
export RUST_LOG="info"

# Profile credentials
export MINIO_ACCESS_KEY="your-key"
export MINIO_SECRET_KEY="your-secret"
```

## Contract Specification

### Contract Structure

```toml
[contract]
name = "dataset_name"           # Required: Contract identifier
version = "0.1.0"              # Required: Semantic version
tags = ["pii", "critical"]     # Optional: Classification tags

[file]                         # Optional: File-level validation
validation = [
  { rule = "row_count", min = 1000, max = 10000 }
]

[[columns]]                    # Required: Column definitions
name = "column_name"
validation = [
  { rule = "not_null" },
  { rule = "unique" },
  { rule = "range", min = 0, max = 100 }
]

[[compound_unique]]            # Optional: Multi-column uniqueness
columns = ["col1", "col2"]

[source]                       # Required: Data source
type = "s3"
location = "s3://bucket/file.csv"
profile = "prod_s3"

[destination]                  # Optional: Success destination
type = "s3"
location = "s3://clean/file.csv"
profile = "prod_s3"

[quarantine]                   # Optional: Failure destination
type = "s3"
location = "s3://quarantine/file.csv"
profile = "prod_s3"
```

### Validation Rules

#### Column Rules
- `not_null`: No null values
- `unique`: All values unique
- `range`: Numeric range validation
- `pattern`: Regex pattern matching
- `max_length`: Maximum string length
- `in_set`: Value must be in specified set
- `not_in_set`: Value must not be in specified set
- `type`: Data type validation
- `outlier_sigma`: Statistical outlier detection
- `date_format`: Date format validation
- `completeness`: Minimum non-null ratio
- `distinctness`: Minimum unique value ratio

#### File Rules
- `row_count`: Minimum/maximum row count
- `completeness`: Overall completeness ratio

#### Compound Rules
- `compound_unique`: Multi-column uniqueness constraints

### Source Types

- **local**: Local file system
- **s3**: Amazon S3 or S3-compatible (MinIO)
- **azure**: Azure Blob Storage *(coming soon)*
- **gcs**: Google Cloud Storage *(coming soon)*
- **sftp**: SFTP servers *(coming soon)*

## Integration Examples

### Airflow
```python
from airflow.operators.bash import BashOperator

validate_task = BashOperator(
    task_id='validate_people',
    bash_command='pipa run people',
    dag=dag
)
```

### GitHub Actions
```yaml
- name: Validate Data Quality
  run: |
    pipa run --all
    if [ $? -ne 0 ]; then
      echo "Data validation failed"
      exit 1
    fi
```

### Python Library
```python
# Coming in next release
import pipeaudit

result = pipeaudit.validate_file("data.csv", "contracts/people.toml")
if result.passed:
    print("Validation passed")
```

## Audit Logging

### Log Format

All validation results are written to structured JSON logs:

```json
{
  "timestamp": "2025-01-25T10:00:00.000Z",
  "event": "validation",
  "contract_name": "people",
  "contract_version": "0.1.0",
  "column": "age",
  "rule": "Range",
  "result": "pass",
  "details": "min=0, max=120"
}
```

### Log Integrity

- Daily log files: `logs/audit-YYYY-MM-DD.jsonl`
- Cryptographic sealing: `logs/hash_ledger.txt`
- Tamper detection via SHA256 hashes
- Compliance-ready audit trail

### Verification

```bash
# Verify yesterday's logs
pipa logs verify

# Verify specific date
pipa logs verify --date 2025-01-24
```

## Architecture

### Core Components

- **Contracts**: TOML-based validation definitions
- **Validators**: Rule implementations (statistical, pattern, etc.)
- **Connectors**: Data source integrations
- **Drivers**: File format parsers (CSV, Parquet, etc.)
- **Engine**: Validation orchestration
- **Logging**: Compliance audit system

### Design Principles

1. **Security First**: No sensitive data exposure
2. **Log-Centric**: Logs are the source of truth
3. **Minimal API**: Simple, focused interfaces
4. **Universal Compatibility**: Works everywhere

## Development

### Prerequisites

- Rust 1.70+
- MinIO or S3 for testing

### Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with logs
RUST_LOG=debug cargo run -- health
```

### Project Structure

```
src/
├── cli.rs              # Command-line interface
├── commands/           # Command implementations
├── contracts/          # Contract parsing and types
├── validators/         # Validation rule implementations
├── connectors/         # Data source connectors
├── drivers/           # File format drivers
├── engine.rs          # Validation orchestration
├── logging.rs         # Audit logging system
├── profiles.rs        # Profile management
└── runner.rs          # Data processing pipeline
```

### Adding Validators

```rust
use crate::validators::{Validator, ValidationReport};
use polars::prelude::*;

pub struct CustomValidator {
    pub threshold: f64,
}

impl Validator for CustomValidator {
    fn name(&self) -> &'static str {
        "CustomRule"
    }
    
    fn validate(&self, series: &Series) -> ValidationResult<ValidationReport> {
        // Implementation here
        Ok(ValidationReport {
            status: "pass",
            details: None,
        })
    }
}
```

## Roadmap

### Current (v0.1)
- ✅ Core validation engine
- ✅ CLI interface
- ✅ S3 and local file support
- ✅ Audit logging with integrity

### Next (v0.2)
- [ ] REST API server
- [ ] Python bindings
- [ ] Additional cloud connectors
- [ ] Streaming validation

### Future (v0.3+)
- [ ] Web UI dashboard
- [ ] ML-based drift detection
- [ ] Advanced scheduling
- [ ] Enterprise SSO

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Areas We Need Help
- Additional data source connectors
- More validation rules
- Performance optimizations
- Documentation improvements

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

- **Documentation**: [docs.pipeaudit.io](https://docs.pipeaudit.io)
- **Issues**: [GitHub Issues](https://github.com/your-org/pipeaudit-core/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-org/pipeaudit-core/discussions)
- **Security**: security@pipeaudit.io

## Acknowledgments

Built with:
- [Polars](https://pola.rs/) - Fast DataFrames for Rust
- [Clap](https://clap.rs/) - Command Line Argument Parser
- [Tokio](https://tokio.rs/) - Asynchronous runtime
- [Serde](https://serde.rs/) - Serialization framework

---

**PipeAudit**: Universal data validation for modern data engineering.