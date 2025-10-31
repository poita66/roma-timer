# Roma Timer Release Guide

This document describes the release process for Roma Timer.

## Automated Release Process

### Triggering a Release

There are two ways to trigger a release:

#### 1. Git Tag (Recommended)
```bash
# Create and push a new version tag
git tag v1.0.0
git push origin v1.0.0
```

#### 2. Manual Workflow Dispatch
1. Go to the [Actions tab](https://github.com/YOUR_USERNAME/roma-timer/actions)
2. Select the "Release" workflow
3. Click "Run workflow"
4. Enter the version tag (e.g., `v1.0.0`)
5. Click "Run workflow"

### What Happens During Release

The release workflow automatically:

1. **Builds Binaries** for multiple platforms:
   - `roma-timer-linux-amd64` - Linux x86_64
   - `roma-timer-linux-arm64` - Linux ARM64
   - `roma-timer-windows-amd64.exe` - Windows x86_64
   - `roma-timer-macos-amd64` - macOS x86_64
   - `roma-timer-macos-arm64` - macOS ARM64 (Apple Silicon)

2. **Builds and Pushes Docker Images** to GitHub Container Registry (GHCR):
   - Multi-architecture support (linux/amd64, linux/arm64)
   - Tagged with version number and `latest`

3. **Creates GitHub Release** with:
   - All platform binaries
   - Docker Compose configuration
   - Release notes with installation instructions

## CI/CD Workflows

### 1. CI Workflow (`.github/workflows/ci.yml`)

**Triggers:**
- Push to `main` or `develop` branches
- Pull requests to `main`

**Jobs:**
- **Test Suite**: Runs tests on stable, beta, and minimum supported Rust versions
- **Docker Build Test**: Validates Docker image builds and runs basic tests
- **Frontend Validation**: Checks HTML, JavaScript syntax, and file sizes
- **Integration Tests**: Tests API endpoints, WebSocket connections, and data persistence
- **Security Scan**: Runs cargo audit and Trivy vulnerability scanner
- **Dependency Check**: Checks for outdated and unused dependencies

### 2. Docker Workflow (`.github/workflows/docker.yml`)

**Triggers:**
- Push to `main` branch
- Manual dispatch

**Jobs:**
- **Build and Push**: Builds and pushes Docker images to GHCR
- **SBOM Generation**: Creates Software Bill of Materials

### 3. Release Workflow (`.github/workflows/release.yml`)

**Triggers:**
- Git tags matching `v*` pattern
- Manual dispatch

**Jobs:**
- **Release Binaries**: Builds platform-specific binaries
- **Docker Release**: Builds and pushes release Docker images
- **Docker Compose Update**: Generates release-specific docker-compose.yml
- **Notify**: Reports overall build status

## Versioning

We follow [Semantic Versioning](https://semver.org/):

- **Major**: Breaking changes
- **Minor**: New features (backward compatible)
- **Patch**: Bug fixes (backward compatible)

### Pre-release Versions

Pre-release versions are supported using suffixes:
- `v1.0.0-alpha.1` - Alpha releases
- `v1.0.0-beta.1` - Beta releases
- `v1.0.0-rc.1` - Release candidates

Pre-releases are marked as such in GitHub releases.

## Docker Images

### Image Locations
- **GHCR**: `ghcr.io/YOUR_USERNAME/roma-timer`
- **Tags**: `v1.0.0`, `latest`, `main-v<sha>`

### Multi-architecture Support
All Docker images support:
- `linux/amd64` - Standard x86_64 servers
- `linux/arm64` - ARM64 servers (AWS Graviton, Raspberry Pi 4)

### Environment Variables

The Docker image supports all the same environment variables as the binary:

```bash
# Data persistence (new recommended approach)
ROMA_TIMER_DATA_DIR=/app/data

# Legacy database path (overrides ROMA_TIMER_DATA_DIR)
DATABASE_URL=/app/data/roma_timer.json

# Server configuration
ROMA_TIMER_HOST=0.0.0.0
ROMA_TIMER_PORT=3000
ROMA_TIMER_SECRET=your-secret-here

# Authentication
ROMA_TIMER_SHARED_SECRET=jwt-secret
ROMA_TIMER_PEPPER=password-pepper
```

## Installation Instructions

### Docker (Recommended)
```bash
docker pull ghcr.io/YOUR_USERNAME/roma-timer:v1.0.0
docker run -p 3000:3000 \
  -v roma-timer-data:/app/data \
  ghcr.io/YOUR_USERNAME/roma-timer:v1.0.0
```

### Docker Compose
```bash
curl -LO https://github.com/YOUR_USERNAME/roma-timer/releases/download/v1.0.0/docker-compose.release.yml
docker compose -f docker-compose.release.yml up -d
```

### Binary
Download the appropriate binary for your platform from the [releases page](https://github.com/YOUR_USERNAME/roma-timer/releases).

## Development Workflow

### Testing Changes
1. Fork the repository
2. Create a feature branch
3. Push changes and create a pull request
4. CI will automatically run tests

### Building Locally
```bash
# Build the backend
cd backend
cargo build --release

# Build Docker image
docker build -t roma-timer:dev .
```

## Troubleshooting

### Common Issues

1. **Docker build failures**: Check that all files are properly committed and that the Dockerfile syntax is valid
2. **Test failures**: Ensure Rust version compatibility and that all dependencies are up to date
3. **Release asset upload failures**: Verify GitHub token permissions
4. **Docker push failures**: Check that you have write permissions to the container registry

### Manual Recovery

If the automated release fails, you can:

1. **Build binaries manually**:
   ```bash
   cargo build --release --target x86_64-unknown-linux-gnu
   ```

2. **Build Docker image manually**:
   ```bash
   docker build -t ghcr.io/YOUR_USERNAME/roma-timer:v1.0.0 .
   docker push ghcr.io/YOUR_USERNAME/roma-timer:v1.0.0
   ```

3. **Create release manually** on GitHub with the built assets

## Security Considerations

- All secrets are stored in GitHub Secrets
- Docker images are scanned for vulnerabilities
- Dependencies are audited for known security issues
- Releases are signed with GitHub's built-in signing

## Questions?

If you have questions about the release process, please:
1. Check this document
2. Look at the workflow files in `.github/workflows/`
3. Open an issue with the "question" label