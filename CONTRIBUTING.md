# Contributing to ARM Hypervisor Platform

Thank you for your interest in contributing to the ARM Hypervisor Platform! This document provides guidelines and information for contributors.

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for all contributors.

## How to Contribute

### Reporting Bugs

1. Check if the bug has already been reported in [Issues](https://github.com/your-org/arm-hypervisor/issues)
2. If not, create a new issue using the bug report template
3. Include:
   - Clear description of the bug
   - Steps to reproduce
   - Expected vs actual behavior
   - Environment details (OS, hardware, version)
   - Relevant logs and configuration

### Suggesting Features

1. Check if the feature has already been requested
2. Create a new issue using the feature request template
3. Explain the use case and expected behavior
4. Discuss the feature with maintainers before implementing

### Pull Requests

1. **Fork the repository**
   ```bash
   git clone https://github.com/your-org/arm-hypervisor.git
   cd arm-hypervisor
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**
   - Write clean, readable code
   - Follow Rust conventions and idioms
   - Add tests for new functionality
   - Update documentation as needed

3. **Test your changes**
   ```bash
   # Run all tests
   cargo test --workspace
   
   # Run clippy
   cargo clippy --all-targets --all-features -- -D warnings
   
   # Format code
   cargo fmt --all
   
   # Security audit
   cargo audit
   ```

4. **Commit your changes**
   ```bash
   git add .
   git commit -m "feat: add new feature description"
   ```
   
   Follow [Conventional Commits](https://www.conventionalcommits.org/):
   - `feat:` - New feature
   - `fix:` - Bug fix
   - `docs:` - Documentation changes
   - `style:` - Code style changes (formatting, etc.)
   - `refactor:` - Code refactoring
   - `test:` - Adding or updating tests
   - `chore:` - Maintenance tasks

5. **Push and create PR**
   ```bash
   git push origin feature/your-feature-name
   ```
   
   Then create a Pull Request on GitHub using the PR template.

## Development Setup

### Prerequisites

- Rust 1.70+ (`rustup` recommended)
- LXC/LXD (for testing container operations)
- Git

### Setting Up Development Environment

```bash
# Clone the repository
git clone https://github.com/your-org/arm-hypervisor.git
cd arm-hypervisor

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install development tools
cargo install cargo-watch cargo-audit

# Build the project
cargo build

# Run tests
cargo test
```

### Development Workflow

```bash
# Watch for changes and rebuild
cargo watch -x build

# Watch and run tests
cargo watch -x test

# Run specific test
cargo test test_name -- --nocapture

# Run integration tests
cargo test --test integration_tests
```

## Code Style

### Rust Guidelines

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Run `cargo clippy` and fix all warnings
- Write documentation comments (`///`) for public items
- Keep functions focused and small
- Prefer descriptive names over comments

### Example

```rust
/// Creates a new LXC container with the specified configuration.
///
/// # Arguments
///
/// * `request` - The container creation request with configuration
///
/// # Returns
///
/// * `Ok(Container)` - Successfully created container
/// * `Err(ContainerError)` - If creation fails
///
/// # Examples
///
/// ```
/// let request = CreateContainerRequest {
///     name: "test-container".to_string(),
///     template: "alpine".to_string(),
///     config: ContainerConfig::default(),
/// };
/// let container = ContainerManager::create(request).await?;
/// ```
pub async fn create(request: CreateContainerRequest) -> Result<Container, ContainerError> {
    // Implementation
}
```

## Testing

### Unit Tests

- Write unit tests for all new functionality
- Place tests in the same file as the code or in `tests/` module
- Use descriptive test names

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_name_validation() {
        assert!(is_valid_name("valid-name"));
        assert!(!is_valid_name(""));
        assert!(!is_valid_name("invalid name"));
    }
}
```

### Integration Tests

- Add integration tests for API endpoints
- Use the test harness in `crates/api-server/tests/`
- Mock external dependencies when possible

## Documentation

### Code Documentation

- Document all public APIs
- Include examples in documentation
- Keep documentation up-to-date with code changes

### User Documentation

- Update `README.md` for user-facing changes
- Update `DEPLOYMENT.md` for deployment changes
- Add examples to documentation

## Commit Guidelines

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Example:**
```
feat(api): add container health check endpoint

Implements GET /api/v1/containers/{id}/health endpoint
that returns container health status including CPU,
memory, and network metrics.

Closes #123
```

### Types

- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation only changes
- `style` - Code style changes (formatting, semicolons, etc.)
- `refactor` - Code change that neither fixes a bug nor adds a feature
- `perf` - Performance improvement
- `test` - Adding or updating tests
- `chore` - Build process or auxiliary tool changes

## Review Process

1. **Automated Checks** - All CI checks must pass
   - Tests pass
   - Clippy warnings resolved
   - Code formatted
   - Security audit clean

2. **Code Review** - At least one maintainer approval required
   - Code quality
   - Tests coverage
   - Documentation completeness
   - Security considerations

3. **Merge** - Maintainers will merge approved PRs

## Project Structure

```
arm-hypervisor/
├── crates/
│   ├── api-server/       # REST API server
│   ├── cluster/          # Clustering logic
│   ├── container-manager/# LXC container management
│   ├── models/           # Shared data models
│   ├── network/          # Network management
│   ├── storage/          # Storage management
│   └── web-ui/           # Web interface
├── scripts/              # Installation and utility scripts
├── tests/                # Integration tests
└── docs/                 # Documentation
```

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create release PR
4. Tag release: `git tag -a v0.x.0 -m "Release v0.x.0"`
5. Push tag: `git push origin v0.x.0`
6. GitHub Actions will build and publish release

## Getting Help

- **Issues**: Check existing issues or create a new one
- **Discussions**: Use GitHub Discussions for questions
- **Documentation**: Read the docs in `/docs` folder

## Recognition

Contributors will be recognized in:
- `CONTRIBUTORS.md` file
- Release notes
- Project documentation

Thank you for contributing to ARM Hypervisor Platform! 🚀
