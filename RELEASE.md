# Release Process

This document describes the release process for ah.

## Versioning

We follow [Semantic Versioning (SemVer)](https://semver.org/):

- **MAJOR** (x.0.0) - Incompatible API changes
- **MINOR** (0.x.0) - New backward-compatible functionality
- **PATCH** (0.0.x) - Backward-compatible bug fixes

## Pre-release Checklist

Before releasing a new version:

- [ ] All tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Security audit passes (`cargo audit`)
- [ ] Changelog is updated with new version
- [ ] Version number updated in `Cargo.toml`
- [ ] Commit changes with version tag

## Release Steps

### 1. Update Version

```bash
# Edit Cargo.toml to bump version
vim Cargo.toml
```

### 2. Update Changelog

Add new version header to `CHANGELOG.md`:

```markdown
## [X.Y.Z] - YYYY-MM-DD

### Added

- New feature description

### Changed

- Change description

### Fixed

- Fix description
```

### 3. Commit Changes

```bash
git add -A
git commit -m "release: vX.Y.Z"
git tag -a vX.Y.Z -m "Version X.Y.Z"
```

### 4. Push to Remote

```bash
git push origin main --tags
```

This will trigger the release workflow automatically.

### 5. Publish to crates.io (Manual)

```bash
cargo publish
```

Or wait for the GitHub Actions workflow to publish automatically.

## GitHub Release

After pushing the tag, GitHub Actions will automatically:

1. Build the release binary
2. Create a GitHub Release with release notes
3. Upload the binary as an artifact
4. Publish to crates.io (if configured)

## crates.io Setup

To enable automatic publishing to crates.io:

1. Get an API token from https://crates.io/settings/tokens
2. Add the token as a secret in GitHub repository settings:
   - Go to: Settings → Secrets and variables → Actions
   - Add new secret: `CARGO_REGISTRY_TOKEN`

## Manual Release (Alternative)

```bash
# Build
cargo build --release

# Login to crates.io
cargo login

# Publish
cargo publish

# Create GitHub release
gh release create vX.Y.Z --title "Version X.Y.Z" --generate-notes
```

## Rollback

If a release has critical issues:

1. yank the crate on crates.io:

   ```bash
   cargo yank --version X.Y.Z
   ```

2. Create a new patch version with the fix

3. Update GitHub release with "This version has been yanked" note
