# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure with workspace configuration
- REST API server with Actix-web
- LXC container management module
- Network bridge and VLAN management
- Storage pool management (local, NFS, CIFS)
- Cluster membership and consensus module
- JWT-based authentication
- TLS/HTTPS support with rustls
- Configuration validation with security checks
- Health check and metrics endpoints
- Systemd service integration
- Comprehensive deployment documentation
- Raspberry Pi bare-metal installation guide
- CI/CD pipeline with GitHub Actions
- Security audit integration
- Integration testing framework

### Security
- JWT secret validation (blocks weak/default secrets)
- Minimum 32-character secret length enforcement
- TLS certificate validation
- CORS origin validation
- Input validation for all API endpoints
- Security audit with cargo-audit

### Changed
- Improved error handling across all modules
- Optimized code for zero clippy warnings
- Enhanced logging with structured output
- Better configuration file examples

### Fixed
- All clippy warnings resolved
- Integration tests now environment-aware
- Redundant closures optimized
- Memory leak prevention in long-running operations

## [0.1.0] - 2026-01-22

### Added
- Initial release
- Core hypervisor functionality
- Container lifecycle management
- Basic clustering support
- RESTful API
- Web UI foundation

[Unreleased]: https://github.com/your-org/arm-hypervisor/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/your-org/arm-hypervisor/releases/tag/v0.1.0
