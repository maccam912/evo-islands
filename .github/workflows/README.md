# GitHub Actions Setup

Due to permission restrictions during development, the GitHub Actions workflow file is stored in this directory.

## To Deploy

Move the workflow file to the standard GitHub Actions location:

```bash
mkdir -p .github/workflows
cp gha/ci.yml .github/workflows/ci.yml
```

Or if you have the right permissions, you can commit the workflow file directly to `.github/workflows/ci.yml`.

## What the Workflow Does

1. **Test**: Runs all tests, formatting checks, and clippy lints on every push and PR
2. **Build Server**: Builds and pushes server Docker image to GitHub Container Registry
3. **Build Client**: Builds and pushes client Docker image to GitHub Container Registry

The workflow automatically runs on:
- Pushes to `main` branch
- Pushes to branches starting with `claude/`
- Pull requests to `main`

## Container Registry

Images are published to GitHub Container Registry (ghcr.io):
- Server: `ghcr.io/maccam912/evo-islands-server:latest`
- Client: `ghcr.io/maccam912/evo-islands-client:latest`

Make sure to enable GitHub Packages in your repository settings and ensure the `GITHUB_TOKEN` has the necessary permissions.
