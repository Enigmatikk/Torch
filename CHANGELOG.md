# Changelog

All notable changes to Torch will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Core web framework with async/await support
- HTTP routing with path parameters
- Middleware system for request/response processing
- Built-in security features (HMAC signing, IP whitelisting, rate limiting)
- Production monitoring (metrics, logging, health checks)
- WebSocket support for real-time applications
- Multi-tier caching (memory and Redis)
- Database connection pooling
- Configuration management via TOML files
- Comprehensive examples and documentation

### Security
- Input validation and sanitization
- CSRF protection
- Security headers middleware
- Request signing with HMAC
- Rate limiting per IP and globally

## [0.1.0] - 2024-01-XX

### Added
- Initial release of Torch web framework
- Basic HTTP server functionality
- Route handling for all HTTP methods
- Middleware support
- JSON serialization support
- Production-ready features
- Security middleware
- Monitoring and metrics
- WebSocket support
- Caching layer
- Database integration
- Configuration system

### Features
- **Performance**: Built on Tokio and Hyper for maximum throughput
- **Security**: Comprehensive security features built-in
- **Production Ready**: Monitoring, logging, and health checks included
- **Real-time**: WebSocket and SSE support
- **Developer Friendly**: Simple, intuitive API design

### Documentation
- Complete API documentation
- Getting started guide
- Production deployment examples
- Security best practices
- Performance tuning guide
