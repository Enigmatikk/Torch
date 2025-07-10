# Contributing to Torch

Thanks for your interest in contributing to Torch! We welcome contributions of all kinds.

## Quick Start

1. Fork the repository
2. Create a feature branch: `git checkout -b my-new-feature`
3. Make your changes
4. Add tests for your changes
5. Run the test suite: `cargo test`
6. Run the examples: `cargo run --example hello_world`
7. Commit your changes: `git commit -am 'Add some feature'`
8. Push to the branch: `git push origin my-new-feature`
9. Submit a pull request

## Development Setup

```bash
# Clone your fork
git clone https://github.com/Enigmatikk/torch.git
cd torch

# Install Rust (if you haven't already)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Run tests
cargo test

# Try the examples
cargo run --example hello_world
cargo run --example production_server --features production
```

## What We're Looking For

### High Priority
- Performance improvements and optimizations
- Security enhancements
- Better error handling and debugging
- More comprehensive tests
- Documentation improvements

### Medium Priority
- New middleware implementations
- Additional database drivers
- More caching backends
- WebSocket enhancements
- API improvements

### Always Welcome
- Bug fixes
- Documentation fixes
- Example improvements
- Performance benchmarks

## Code Style

- Follow standard Rust formatting: `cargo fmt`
- Run clippy: `cargo clippy`
- Write tests for new functionality
- Update documentation for public APIs
- Keep commits focused and atomic

## Testing

```bash
# Run all tests
cargo test

# Run tests with all features
cargo test --all-features

# Run specific test
cargo test test_name

# Run examples to verify they work
cargo run --example hello_world
cargo run --example rest_api --features json
```

## Documentation

- Update README.md if you add new features
- Add doc comments for public APIs
- Include examples in doc comments when helpful
- Update CHANGELOG.md for notable changes

## Pull Request Guidelines

- Keep PRs focused on a single feature or fix
- Include tests for new functionality
- Update documentation as needed
- Ensure all tests pass
- Follow the existing code style
- Write clear commit messages

## Reporting Issues

When reporting bugs, please include:
- Rust version (`rustc --version`)
- Torch version
- Operating system
- Minimal code example that reproduces the issue
- Expected vs actual behavior

## Feature Requests

We love hearing about new ideas! When suggesting features:
- Explain the use case and why it's valuable
- Consider if it fits with Torch's goals (performance, security, simplicity)
- Think about backwards compatibility
- Provide examples of how the API might look

## Code of Conduct

Be respectful, inclusive, and constructive. We want Torch to be a welcoming project for everyone.

## Questions?

- Open an issue for bugs or feature requests
- Start a discussion for questions about usage
- Check existing issues before creating new ones

Thanks for contributing! 🔥
