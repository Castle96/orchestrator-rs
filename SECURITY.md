# Security Policy

## Supported Versions

We release patches for security vulnerabilities for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via email to: [security@your-domain.com]

You should receive a response within 48 hours. If for some reason you do not, please follow up via email to ensure we received your original message.

Please include the following information (as much as you can provide):

- Type of issue (e.g., buffer overflow, SQL injection, cross-site scripting, etc.)
- Full paths of source file(s) related to the manifestation of the issue
- The location of the affected source code (tag/branch/commit or direct URL)
- Any special configuration required to reproduce the issue
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact of the issue, including how an attacker might exploit it

This information will help us triage your report more quickly.

## Security Best Practices

### For Deployment

1. **JWT Secrets**
   - Never use default or weak JWT secrets
   - Generate secrets with `openssl rand -base64 32`
   - Store secrets in environment variables, not config files
   - Rotate secrets regularly (every 90 days recommended)

2. **TLS/HTTPS**
   - Always use TLS in production
   - Use certificates from trusted CAs
   - Keep certificates up to date
   - Use strong cipher suites

3. **Network Security**
   - Use firewall rules to restrict access
   - Only expose necessary ports
   - Use private networks for cluster communication
   - Implement rate limiting

4. **Access Control**
   - Use strong passwords
   - Enable SSH key authentication
   - Disable root login
   - Use sudo for privileged operations

5. **Updates**
   - Keep system packages updated
   - Run `cargo audit` regularly
   - Subscribe to security advisories
   - Test updates in staging before production

### For Development

1. **Code Review**
   - All code changes require review
   - Security-sensitive changes require security review
   - Run `cargo clippy` and fix all warnings
   - Run `cargo audit` before merging

2. **Dependencies**
   - Review all dependencies before adding
   - Keep dependencies updated
   - Monitor for security advisories
   - Remove unused dependencies

3. **Testing**
   - Write security tests
   - Test authentication and authorization
   - Test input validation
   - Perform penetration testing

## Security Features

### Current

- JWT-based authentication
- TLS/HTTPS support
- Input validation
- CORS protection
- Rate limiting support
- Secure default configurations

### Planned

- Role-based access control (RBAC)
- API key rotation
- Audit logging
- Intrusion detection
- Two-factor authentication

## Security Audit

We use `cargo audit` to check for known vulnerabilities:

```bash
# Install cargo-audit
cargo install cargo-audit

# Run audit
cargo audit

# Run in CI/CD
cargo audit --deny warnings
```

## Disclosure Policy

When we receive a security bug report, we will:

1. Confirm the problem and determine affected versions
2. Audit code to find similar problems
3. Prepare fixes for all supported versions
4. Release new versions as soon as possible
5. Publish a security advisory

## Known Issues

### Current Advisories

- **RUSTSEC-2025-0134**: rustls-pemfile is unmaintained
  - **Severity**: LOW
  - **Status**: Monitoring for maintained alternative
  - **Impact**: Minimal - PEM parsing is stable
  - **Mitigation**: Planning migration to maintained fork

## Security Contacts

- Security Email: [security@your-domain.com]
- GPG Key: [Your GPG Key ID]

## Acknowledgments

We thank the following security researchers for their contributions:

- [Future contributors will be listed here]

## References

- [RustSec Advisory Database](https://rustsec.org/)
- [OWASP Top Ten](https://owasp.org/www-project-top-ten/)
- [Rust Security Guidelines](https://doc.rust-lang.org/cargo/guide/security.html)
