# 2026-01-18: Upload packages to PyPI

Owner: Will Lachance <wlach@protonmail.com>

## Overview

### Problem Statement

Save maintainer time and avoid security gotchas by using github actions to upload to pypi

### Context (as needed)

This repo ships a Rust CLI as a Python package via `maturin` (`bindings = "bin"`). CI already builds wheels on Linux/macOS/Windows and runs a smoke test from the built wheel.

### Goals

- Make it easier to publish new releases
- Improve security

### Non-Goals

- Automate version bumping (will still entail a seperate commit to bump version)

### Proposed Solution

Use the idiomatic best practice way of doing this, using the official packaging guide:

https://packaging.python.org/en/latest/guides/publishing-package-distribution-releases-using-github-actions-ci-cd-workflows/

In general this means adding a new GitHub workflow which ties uploads to PyPI to new releases, using PyPI Trusted Publishers (OIDC) rather than long-lived API tokens.

**Scope**

- Project: `idr-tools` (single PyPI project)
- Artifacts: wheels (Linux/macOS/Windows) and sdist

**Release trigger**

- Publish on GitHub Release published for a `vX.Y.Z` tag.
- Build and test first; publish only if CI passes.

**Security posture**

- Use `pypa/gh-action-pypi-publish` with `id-token: write` (OIDC).
- Configure PyPI Trusted Publisher for this repo and workflow name.

**Keep it simple**

- Publish to TestPyPI on every merge to `main` (allow it to fail if the version already exists).
- Use `continue-on-error: true` on the TestPyPI publish step.
- No automated version bumping, changelog, or release notes.

## Other reading (as needed)

- [Official Guide](https://packaging.python.org/en/latest/guides/publishing-package-distribution-releases-using-github-actions-ci-cd-workflows/)
- [Stamina's implementation](https://github.com/hynek/stamina/blob/b25d4bc359ff603496aafbb217ab82c5a43715a6/.github/workflows/pypi-package.yml) (likely best practice)

## Implementation (ephemeral)

1. Add a new GitHub Actions workflow (e.g., `.github/workflows/publish-pypi.yml`) that:
   - Builds wheels and sdist with `maturin` (Linux/macOS/Windows).
   - Uploads artifacts to the workflow for reuse by publish jobs.
2. Add a TestPyPI publish job:
   - Trigger on pushes to `main`.
   - Download artifacts and publish with `pypa/gh-action-pypi-publish` using OIDC.
   - Mark the publish step `continue-on-error: true`.
3. Add a PyPI publish job:
   - Trigger on `release: published` for `vX.Y.Z` tags.
   - Download artifacts and publish with `pypa/gh-action-pypi-publish` using OIDC.
4. Configure Trusted Publishers in TestPyPI and PyPI:
   - Repo: `wlach/idr-tools`
   - Workflow name: `Build & publish to PyPI` (or the chosen workflow name).
   - Environment names: `release-test-pypi`, `release-pypi`.
5. Dry run:
   - Merge to `main` and verify TestPyPI publish behavior (allowed to fail if version exists).
   - Cut a GitHub Release for `v0.1.0` and confirm PyPI publish.
