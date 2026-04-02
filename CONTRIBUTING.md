# Contributing to ah

Thank you for your interest in contributing to ah! We welcome various forms of contributions, including bug reports, feature requests, code submissions, documentation improvements, and more.

## Code of Conduct

Please read and follow our [Code of Conduct](https://www.contributor-covenant.org/) to maintain a friendly and professional environment.

## How to Contribute

### Reporting Bugs

1. Search existing [Issues](https://github.com/your-repo/ah/issues) to avoid duplicates
2. Use the Bug Report template to create a new issue
3. Include the following information:
   - Steps to reproduce
   - Expected vs actual behavior
   - Environment info (OS, Nix version, Rust version)
   - Relevant log output

### Requesting Features

1. Search existing Issues and PRs
2. Use the Feature Request template to create a new issue
3. Clearly describe the use case and expected behavior

### Code Contributions

#### Development Environment

```bash
# Clone the project
git clone https://github.com/z1-0/ah.sh.git
cd ah

# Enter development environment
nix develop

# Or use cargo
cargo build
```

#### Code Standards

- Format code with `cargo fmt`
- Run `cargo clippy` to check for warnings
- Keep code concise, following Rust functional programming patterns
- Use the `fp-core` library for functional utilities

#### Commit Convention

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>: <description>

[optional body]

[optional footer]
```

Types:

- `feat`: New feature
- `fix`: Bug fix
- `refactor`: Code refactoring
- `docs`: Documentation updates
- `style`: Code style (formatting, no functionality change)
- `test`: Test related
- `chore`: Build process or tooling changes

Example:

```
feat(session): add session restore by index

Add ability to restore sessions using index number
in addition to session ID.

Closes #123
```

#### Pull Request Process

1. Fork this repository
2. Create a feature branch: `git checkout -b feat/my-feature`
3. Make your changes and commit
4. Push branch: `git push -u origin feat/my-feature`
5. Create a Pull Request
6. Wait for code review and merge

#### PR Requirements

- [ ] All tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Include appropriate tests (if applicable)
- [ ] Update relevant documentation

### Documentation Contributions

- Fix spelling and grammar errors
- Improve clarity of existing documentation
- Translate documentation to other languages
- Add missing examples

## Development Guide

### Project Structure

```
src/
├── cli/          # Command-line parsing
├── provider/     # Provider abstraction and implementation
├── session/      # Session management
├── cmd.rs        # Shell command execution
├── manager.rs    # Core business orchestration
└── paths.rs      # Path utilities

```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test <test_name>

# Run tests in release mode
cargo test --release
```

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run locally
cargo run -- --help
cargo run -- use rust go
```

## License

By contributing code, you agree to release your contributions under the project's license (MIT).
