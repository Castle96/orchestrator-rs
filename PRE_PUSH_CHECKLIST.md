# Pre-Push Checklist

Use this checklist before pushing to GitHub to ensure your repository is ready.

## ✅ Code Quality

- [ ] All tests pass: `cargo test --workspace`
- [ ] Integration tests pass: `cargo test --test integration_tests`
- [ ] No clippy warnings: `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] Code is formatted: `cargo fmt --all -- --check`
- [ ] Security audit clean: `cargo audit` (or acceptable warnings documented)
- [ ] No debug print statements or commented-out code
- [ ] All TODOs have issue numbers or are removed

## ✅ Documentation

- [ ] README.md is up-to-date with current features
- [ ] All public APIs have documentation comments
- [ ] CHANGELOG.md updated with latest changes
- [ ] Example configuration files are current
- [ ] Deployment guide reflects actual installation process
- [ ] Security policy is accurate

## ✅ Configuration

- [ ] No secrets or sensitive data in config files
- [ ] Example configs use placeholder values
- [ ] .gitignore includes all necessary patterns
- [ ] Environment variables documented
- [ ] Default values are secure

## ✅ GitHub Setup

- [ ] Update repository URL in README.md badges
- [ ] Update security contact email in SECURITY.md
- [ ] Update repository links in CONTRIBUTING.md
- [ ] Verify LICENSE files are correct
- [ ] Add repository description and topics
- [ ] Enable security features (Dependabot, secret scanning)

## ✅ CI/CD

- [ ] GitHub Actions workflows are present
- [ ] Workflows are configured correctly
- [ ] Required secrets are documented (if any)
- [ ] Build matrix covers target platforms

## ✅ Final Steps

1. **Update Repository URLs**
   - [ ] Replace `your-org` in README.md
   - [ ] Replace `your-domain.com` in SECURITY.md
   - [ ] Update contributor email/info

2. **Clean Build**
   ```bash
   cargo clean
   cargo build --release
   cargo test --workspace
   ```

3. **Git Status**
   ```bash
   # Verify nothing important is ignored
   git status
   
   # Review all changes
   git diff
   ```

4. **Initial Commit**
   ```bash
   git add .
   git commit -m "feat: initial release of ARM Hypervisor Platform
   
   - Complete LXC container management
   - High-availability clustering
   - TLS/HTTPS security
   - Production-ready configuration
   - Comprehensive documentation
   - CI/CD pipeline
   - Integration tests"
   ```

5. **Push to GitHub**
   ```bash
   # Create GitHub repository first, then:
   git remote add origin https://github.com/your-org/arm-hypervisor.git
   git branch -M main
   git push -u origin main
   ```

6. **Post-Push Setup**
   - [ ] Enable GitHub Actions
   - [ ] Configure branch protection rules (require PR reviews, passing CI)
   - [ ] Enable Dependabot alerts
   - [ ] Enable security scanning
   - [ ] Add repository topics (rust, raspberry-pi, lxc, virtualization, arm64)
   - [ ] Create initial GitHub release (v0.1.0)
   - [ ] Pin important issues
   - [ ] Configure GitHub Pages (if using)

## ✅ Optional Enhancements

- [ ] Add badges to README (build status, coverage, downloads)
- [ ] Set up code coverage reporting (codecov.io, coveralls)
- [ ] Configure release automation
- [ ] Add contributing guide badge
- [ ] Set up issue templates
- [ ] Create project board for roadmap
- [ ] Add social preview image

## Common Issues to Check

### Security
- No API keys, passwords, or tokens in code
- No default credentials that could be used in production
- All example configs clearly marked as examples
- TLS certificates not committed

### Performance
- No development/debug code in release builds
- Async operations properly handled
- Resource cleanup in destructors

### Compatibility
- Rust version requirement documented
- System dependencies listed
- Target platforms specified
- Breaking changes documented

## Quick Commands

```bash
# Full pre-push check
cargo clean && \
cargo build --release && \
cargo test --workspace && \
cargo clippy --all-targets --all-features -- -D warnings && \
cargo fmt --all -- --check && \
cargo audit

# If all pass:
echo "✅ Ready to push!"
```

## After First Push

1. **Monitor CI Pipeline**
   - Check GitHub Actions results
   - Fix any environment-specific failures
   - Verify release artifacts are built

2. **Community Setup**
   - Star your own repo (shows it's active!)
   - Write introduction in Discussions
   - Share on relevant communities (r/rust, r/raspberry_pi, etc.)
   - Tag with #rustlang, #raspberrypi on social media

3. **Documentation**
   - Add quickstart guide to wiki
   - Create FAQ section
   - Document common troubleshooting steps

---

**Remember**: Once pushed to public GitHub, assume everything is permanent. Triple-check for secrets!
