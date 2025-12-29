# Contributing to dot

Thank you for your interest in contributing to `dot`! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Code Style](#code-style)
- [Documentation](#documentation)
- [Release Process](#release-process)

## Code of Conduct

This project adheres to a code of conduct that we expect all contributors to follow. Please be respectful and constructive in all interactions.

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) 1.70 or later
- [Git](https://git-scm.com/)
- GitHub account with a personal access token

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone git@github.com:yourusername/dot.git
   cd dot
   ```

3. Add the upstream remote:
   ```bash
   git remote add upstream git@github.com:username/dot.git
   ```

## Development Setup

### Initial Setup

```bash
# Install dependencies and run initial tests
make setup

# Verify everything works
make test
make build
```

### Development Environment

```bash
# Install development dependencies
cargo install cargo-watch
cargo install cargo-audit
cargo install cargo-outdated

# Set up pre-commit hooks (optional)
make install-hooks
```

### Configuration for Development

Create a development configuration:

```bash
# Create test configuration
mkdir -p ~/.dot
echo '{
  "authorized_organizations": ["your-test-org"],
  "default_organization": "your-test-org"
}' > ~/.dot/dot.conf

# Set GitHub token for testing
export GITHUB_TOKEN="your_github_token_here"
```

## Making Changes

### Branch Naming

Use descriptive branch names:
- `feature/add-new-command` - for new features
- `fix/handle-edge-case` - for bug fixes
- `docs/update-readme` - for documentation changes
- `refactor/improve-error-handling` - for refactoring

### Commit Messages

Follow conventional commit format:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: new feature
- `fix`: bug fix
- `docs`: documentation changes
- `style`: formatting, missing semicolons, etc.
- `refactor`: code change that neither fixes a bug nor adds a feature
- `test`: adding missing tests
- `chore`: maintain

Examples:
```
feat(cli): add support for multiple hidden directories

fix(atomic): handle rollback failure gracefully

docs(readme): add troubleshooting section
```

### Development Workflow

1. **Create a branch:**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes:**
   ```bash
   # Edit files
   # Test your changes
   make test
   ```

3. **Run quality checks:**
   ```bash
   make fmt      # Format code
   make clippy   # Run linter
   make test     # Run tests
   make audit    # Security audit
   ```

4. **Commit your changes:**
   ```bash
   git add .
   git commit -m "feat(scope): your descriptive message"
   ```

5. **Keep your branch updated:**
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

## Testing

### Running Tests

```bash
# Run all tests
make test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run integration tests
cargo test --test integration

# Run with coverage
make coverage
```

### Writing Tests

#### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_key_generation() {
        let url = "git@github.com:user/repo.git";
        let dir = ".kiro";
        let expected = "github.com/user/repo/.kiro";
        
        assert_eq!(generate_repository_key(url, dir), expected);
    }
}
```

#### Integration Tests

Create files in `tests/` directory:

```rust
// tests/integration_test.rs
use dot::config::ConfigManager;

#[tokio::test]
async fn test_config_loading() {
    // Test configuration loading
}
```

### Test Guidelines

- Write tests for all new functionality
- Include edge cases and error conditions
- Use descriptive test names
- Mock external dependencies (GitHub API, file system)
- Test both success and failure paths

## Submitting Changes

### Pull Request Process

1. **Ensure your branch is up to date:**
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run all checks:**
   ```bash
   make check-all
   ```

3. **Push your branch:**
   ```bash
   git push origin feature/your-feature-name
   ```

4. **Create a Pull Request:**
   - Use a descriptive title
   - Fill out the PR template
   - Link any related issues
   - Add screenshots for UI changes

### Pull Request Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Tests pass locally
- [ ] Added tests for new functionality
- [ ] Manual testing completed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No new warnings introduced
```

### Review Process

- All PRs require at least one review
- Address all review comments
- Keep PRs focused and reasonably sized
- Be responsive to feedback

## Code Style

### Rust Style Guidelines

We follow standard Rust conventions:

```bash
# Format code
cargo fmt

# Check style
cargo clippy

# Fix clippy warnings
cargo clippy --fix
```

### Specific Guidelines

1. **Error Handling:**
   ```rust
   // Use thiserror for error types
   #[derive(Error, Debug)]
   pub enum DotError {
       #[error("Configuration error: {0}")]
       Config(String),
   }
   ```

2. **Async Functions:**
   ```rust
   // Use async/await consistently
   pub async fn load_config() -> Result<Config, DotError> {
       // implementation
   }
   ```

3. **Documentation:**
   ```rust
   /// Generates a repository key from URL and directory
   /// 
   /// # Arguments
   /// * `url` - Git remote URL
   /// * `directory` - Hidden directory name
   /// 
   /// # Returns
   /// Unique repository key string
   pub fn generate_repository_key(url: &str, directory: &str) -> String {
       // implementation
   }
   ```

### Code Organization

```
src/
├── main.rs          # CLI entry point
├── lib.rs           # Library exports
├── config.rs        # Configuration management
├── index.rs         # Global index operations
├── repository.rs    # Repository operations
├── atomic.rs        # Atomic operation framework
├── git_operations.rs # Git command wrappers
└── error.rs         # Error types
```

## Documentation

### Types of Documentation

1. **Code Documentation:**
   - Document all public functions
   - Include examples in doc comments
   - Use `cargo doc` to generate docs

2. **User Documentation:**
   - Update README.md for new features
   - Add examples to docs/examples.md
   - Update troubleshooting guide

3. **Developer Documentation:**
   - Update this CONTRIBUTING.md
   - Document architecture decisions
   - Maintain changelog

### Documentation Standards

```rust
/// Brief description of the function
///
/// Longer description if needed, explaining the purpose,
/// behavior, and any important details.
///
/// # Arguments
///
/// * `param1` - Description of parameter
/// * `param2` - Description of parameter
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// Description of possible errors
///
/// # Examples
///
/// ```
/// use dot::config::ConfigManager;
/// 
/// let config = ConfigManager::load().await?;
/// ```
pub async fn example_function(param1: &str, param2: i32) -> Result<String, DotError> {
    // implementation
}
```

## Release Process

### Version Numbering

We use [Semantic Versioning](https://semver.org/):
- `MAJOR.MINOR.PATCH`
- Major: breaking changes
- Minor: new features (backward compatible)
- Patch: bug fixes (backward compatible)

### Release Checklist

1. **Update version numbers:**
   ```bash
   # Update Cargo.toml
   # Update README.md badges
   # Update documentation
   ```

2. **Update changelog:**
   ```bash
   # Add new version section to CHANGELOG.md
   # List all changes since last release
   ```

3. **Create release:**
   ```bash
   git tag v1.2.3
   git push upstream v1.2.3
   ```

4. **Publish:**
   ```bash
   cargo publish
   ```

## Getting Help

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and ideas
- **Pull Request Comments**: Code-specific discussions

### Development Questions

If you have questions about:
- Architecture decisions
- Implementation approaches
- Testing strategies
- Code organization

Feel free to:
1. Open a GitHub Discussion
2. Comment on related issues
3. Ask in your pull request

### Useful Commands

```bash
# Development workflow
make setup          # Initial setup
make build          # Build debug version
make test           # Run tests
make fmt            # Format code
make clippy         # Run linter
make check-all      # Run all checks

# Release workflow
make release        # Build release version
make install        # Install locally
make clean          # Clean build artifacts

# Documentation
cargo doc --open    # Generate and open docs
make docs           # Build all documentation
```

Thank you for contributing to `dot`! Your contributions help make this tool better for everyone.