# Publishing Guide - Hope Genome

Complete guide for publishing Hope Genome to PyPI and Crates.io.

## 📦 PyPI Publishing

### Prerequisites

1. **Create PyPI Account**: https://pypi.org/account/register/
2. **Create API Token**:
   - Go to https://pypi.org/manage/account/token/
   - Create token with "Entire account" scope
   - Save the token securely (starts with `pypi-`)

### GitHub Secrets Setup

1. Go to GitHub repository → Settings → Secrets and variables → Actions
2. Click "New repository secret"
3. Add secret:
   - Name: `PYPI_API_TOKEN`
   - Value: `pypi-...` (your token)

### Automated Publishing

The `.github/workflows/wheels.yml` workflow automatically:
- Builds wheels for all platforms (Linux x86_64/aarch64, Windows, macOS Intel/ARM)
- Builds source distribution (sdist)
- Publishes to PyPI on every tag push

**Trigger a release:**
```bash
git tag v1.5.1
git push origin v1.5.1
```

### Manual Publishing (if needed)

```bash
# Install maturin
pip install maturin

# Build wheels
maturin build --release --features python-bindings

# Upload to PyPI
maturin publish --username __token__ --password pypi-YOUR_TOKEN_HERE
```

### Test on TestPyPI First

```bash
# Build
maturin build --release --features python-bindings

# Upload to TestPyPI
maturin publish --repository testpypi --username __token__ --password pypi-YOUR_TESTPYPI_TOKEN

# Test installation
pip install --index-url https://test.pypi.org/simple/ hope-genome
```

## 🦀 Crates.io Publishing

### Prerequisites

1. **Create Crates.io Account**: https://crates.io/
2. **Get API Token**: https://crates.io/me
3. **Login locally**:
   ```bash
   cargo login YOUR_TOKEN_HERE
   ```

### Publishing

```bash
cd hope_core

# Dry run (verify everything)
cargo publish --dry-run

# Publish to crates.io
cargo publish
```

### Automation (Optional)

Add to `.github/workflows/release.yml`:

```yaml
- name: Publish to crates.io
  run: |
    cd hope_core
    cargo publish --token ${{ secrets.CARGO_REGISTRY_TOKEN }}
```

**GitHub Secret:**
- Name: `CARGO_REGISTRY_TOKEN`
- Value: Your crates.io token

## 🐳 Docker Hub Publishing

### Prerequisites

1. **Create Docker Hub Account**: https://hub.docker.com/
2. **Create Access Token**: Settings → Security → New Access Token

### GitHub Secrets

- Name: `DOCKER_USERNAME` (your Docker Hub username)
- Name: `DOCKER_PASSWORD` (your access token)

### Automated Publishing

See `.github/workflows/docker.yml` (TODO)

### Manual Publishing

```bash
# Build image
docker build -t hope-genome:1.5.0 .
docker tag hope-genome:1.5.0 yourusername/hope-genome:1.5.0
docker tag hope-genome:1.5.0 yourusername/hope-genome:latest

# Login
docker login

# Push
docker push yourusername/hope-genome:1.5.0
docker push yourusername/hope-genome:latest
```

## 📚 Documentation Site (GitHub Pages)

### Setup

1. GitHub repository → Settings → Pages
2. Source: "GitHub Actions"
3. Workflow will auto-deploy on push to main

### Manual Build

```bash
cd hope_core
cargo doc --no-deps --features python-bindings
```

Documentation will be at `target/doc/hope_core/index.html`

## 🔄 Release Checklist

Before releasing a new version:

- [ ] Update version in `hope_core/Cargo.toml`
- [ ] Update version in `pyproject.toml`
- [ ] Update `CHANGELOG.md`
- [ ] Run full test suite: `cargo test --all-features`
- [ ] Run security audit: `cargo audit`
- [ ] Update `RELEASE_NOTES_vX.Y.Z.md`
- [ ] Commit changes: `git commit -am "Release vX.Y.Z"`
- [ ] Tag release: `git tag vX.Y.Z`
- [ ] Push: `git push && git push origin vX.Y.Z`
- [ ] Wait for CI to complete (wheels build + PyPI upload)
- [ ] Create GitHub Release with binaries
- [ ] Publish to crates.io: `cd hope_core && cargo publish`
- [ ] Update documentation site
- [ ] Announce on social media / forums

## 🎯 Version Management

### Semantic Versioning

Hope Genome follows [SemVer](https://semver.org/):

- **MAJOR** (1.x.x): Breaking API changes
- **MINOR** (x.5.x): New features, backward compatible
- **PATCH** (x.x.1): Bug fixes, backward compatible

### Version Bump Script

```bash
# Bump patch version
./scripts/bump_version.sh patch

# Bump minor version
./scripts/bump_version.sh minor

# Bump major version
./scripts/bump_version.sh major
```

## 🔐 Security

### API Token Security

- **NEVER** commit tokens to git
- Use GitHub Secrets for automation
- Rotate tokens regularly
- Use scoped tokens (specific packages only)

### Signing Releases

```bash
# Sign git tag
git tag -s v1.5.0 -m "Release v1.5.0"

# Verify signature
git tag -v v1.5.0
```

## 📊 Monitoring

### PyPI Statistics

- **Downloads**: https://pypistats.org/packages/hope-genome
- **Package page**: https://pypi.org/project/hope-genome/

### Crates.io Statistics

- **Downloads**: https://crates.io/crates/hope_core
- **Version history**: https://crates.io/crates/hope_core/versions

### GitHub Statistics

- **Releases**: https://github.com/silentnoisehun/Hope_Genome/releases
- **Traffic**: Repository → Insights → Traffic

## 🆘 Troubleshooting

### PyPI Upload Fails

**Error**: "File already exists"
- Solution: Use `--skip-existing` flag or bump version

**Error**: "Invalid token"
- Solution: Regenerate token, update GitHub Secret

### Crates.io Upload Fails

**Error**: "crate version already exists"
- Solution: Bump version in Cargo.toml

**Error**: "token not valid"
- Solution: Run `cargo login` with fresh token

### Wheel Build Fails

**Error**: "Python version not supported"
- Solution: Check PyO3 compatibility matrix
- Set `PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1` if needed

## 📞 Support

- **Issues**: https://github.com/silentnoisehun/Hope_Genome/issues
- **Discussions**: https://github.com/silentnoisehun/Hope_Genome/discussions
- **Email**: stratosoiteam@gmail.com
