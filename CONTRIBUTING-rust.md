# Contributing to ADK Rust

We welcome contributions to the Agent Development Kit (ADK) Rust implementation! This document provides guidelines for contributing to the project.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/your-username/adk-rust.git
   cd adk-rust
   ```
3. **Install Rust** if you haven't already:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
4. **Build the project**:
   ```bash
   cargo build
   ```
5. **Run tests**:
   ```bash
   cargo test
   ```

## Development Workflow

1. **Create a new branch** for your feature or bug fix:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** following the coding standards below

3. **Add tests** for your changes

4. **Run the test suite**:
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

5. **Commit your changes** with a clear commit message:
   ```bash
   git commit -m "Add feature: description of your changes"
   ```

6. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

7. **Create a Pull Request** on GitHub

## Coding Standards

### Rust Style Guide

- Follow the [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/)
- Use `cargo fmt` to format your code
- Use `cargo clippy` to catch common mistakes
- Write documentation for public APIs using `///` comments
- Add examples in documentation when appropriate

### Code Organization

- Keep modules focused and cohesive
- Use clear, descriptive names for functions and variables
- Prefer composition over inheritance
- Use `Result<T>` for error handling
- Use `async/await` for asynchronous operations

### Testing

- Write unit tests for all public functions
- Write integration tests for complex workflows
- Use descriptive test names that explain what is being tested
- Test both success and error cases
- Use `#[tokio::test]` for async tests

### Documentation

- Document all public APIs
- Include examples in documentation
- Keep README.md up to date
- Add inline comments for complex logic

## Project Structure

```
src/
├── lib.rs              # Main library entry point
├── main.rs             # CLI binary entry point
├── error.rs            # Error types
├── types.rs            # Common types
├── agents/             # Agent system
├── models/             # LLM models
├── tools/              # Tool system
├── events/             # Event system
├── sessions/           # Session management
├── memory/             # Memory system
├── artifacts/          # Artifact management
├── evaluation/         # Evaluation system
├── runners.rs          # Agent runners
├── cli/                # CLI commands
├── web/                # Web server
└── utils/              # Utilities
```

## Adding New Features

### Adding a New Agent Type

1. Create a new file in `src/agents/`
2. Implement the `BaseAgent` trait
3. Add appropriate tests
4. Update the module exports in `src/agents/mod.rs`
5. Add documentation and examples

### Adding a New Tool

1. Create a new file in `src/tools/`
2. Implement the `BaseTool` trait
3. Add function declaration generation
4. Add tests for the tool
5. Update the module exports in `src/tools/mod.rs`
6. Add documentation and examples

### Adding a New Model

1. Create a new file in `src/models/`
2. Implement the `BaseLlm` trait
3. Add appropriate configuration options
4. Add tests for the model
5. Update the module exports in `src/models/mod.rs`
6. Consider adding a feature flag for optional dependencies

## Pull Request Guidelines

- **Title**: Use a clear, descriptive title
- **Description**: Explain what your PR does and why
- **Testing**: Describe how you tested your changes
- **Breaking Changes**: Clearly mark any breaking changes
- **Documentation**: Update documentation if needed

### PR Checklist

- [ ] Code follows the style guidelines
- [ ] Tests pass locally
- [ ] New tests added for new functionality
- [ ] Documentation updated
- [ ] No breaking changes (or clearly marked)
- [ ] Commit messages are clear and descriptive

## Reporting Issues

When reporting issues, please include:

- **Rust version**: Output of `rustc --version`
- **ADK version**: Version of the ADK you're using
- **Operating system**: OS and version
- **Description**: Clear description of the issue
- **Reproduction steps**: Steps to reproduce the issue
- **Expected behavior**: What you expected to happen
- **Actual behavior**: What actually happened
- **Code sample**: Minimal code that reproduces the issue

## Feature Requests

When requesting features:

- **Use case**: Describe your use case
- **Proposed solution**: Suggest how it might work
- **Alternatives**: Consider alternative approaches
- **Impact**: Describe the impact on existing code

## Code of Conduct

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). Please be respectful and inclusive in all interactions.

## License

By contributing to this project, you agree that your contributions will be licensed under the Apache 2.0 License.

## Questions?

If you have questions about contributing, feel free to:

- Open an issue for discussion
- Ask in the project discussions
- Reach out to the maintainers

Thank you for contributing to ADK Rust!
