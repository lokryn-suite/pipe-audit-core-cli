# Changelog
All notable changes to this project will be documented in this file.

## [0.2.1] - 2025-11-10
### Changed
- Version bump for patch release
- Optimized release build for deployment

## [0.2.0] - 2025-11-10
### Added
- Pluggable logging infrastructure with `AuditLogger` trait
- `JsonlLogger` implementation for JSONL-based audit logging
- `NoOpLogger` implementation for testing and scenarios without logging
- New `audit_logging` public module exposing logging API
- Dependency injection pattern for logger implementations

### Changed
- Refactored logging system to use trait-based architecture
- Updated all engine functions to accept generic `AuditLogger` parameter
- Renamed public logging module from `logging` to `audit_logging` to avoid naming conflicts
- All CLI commands now accept and pass logger implementations to engine functions

### Internal
- Kept existing JSONL logging implementation as default
- Maintained backward compatibility with existing audit log schema
- Improved testability through dependency injection

## [0.1.3] - 2025-10-03
### Added
- Linked full documentation at https://docs.lokryn.com
- Added Discord community link
- Example contracts and profiles for contributors
- Initial integration test scaffold

## [0.1.2] - 2025-09-20
### Fixed
- Corrected validator edge cases for `not_null` and `range`
- Added Change log
