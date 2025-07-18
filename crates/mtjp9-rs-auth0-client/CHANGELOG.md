# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-07-18

### Added

- Initial release of mtjp9-rs-auth0-client - a Rust client library for Auth0 Management API v2
- **OAuth Token Management**
  - Client Credentials Grant support for machine-to-machine authentication
  - Authorization Code Grant with optional PKCE support
  - Refresh Token Grant for token renewal
  - Comprehensive token request/response handling
- **Organization Management**
  - Create organizations with full configuration (branding, metadata, connections)
  - Update organizations with partial updates (patch_organization)
- **User Management**
  - Create users with comprehensive profile information
  - Builder pattern for constructing user creation requests
  - Support for email/phone verification, metadata, and blocked status
- **Ticket Management**
  - Password change ticket generation with customizable TTL
  - Support for result URL redirection and email verification
  - Builder pattern for flexible ticket creation
- **Core Infrastructure**
  - Type-safe domain and bearer token handling
  - Comprehensive error handling for all Auth0 API responses
  - Automatic token redaction in debug output for security
  - Async/await support with tokio
- **Developer Experience**
  - Builder patterns for complex request types
  - Extensive unit tests with mockito integration
  - Random password generation utility (64 characters)
  - Full serde integration for JSON serialization
