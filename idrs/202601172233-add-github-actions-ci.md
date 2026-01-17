# 2026-01-17: Add GitHub Actions CI

Owner: Will Lachance <wlach@protonmail.com>

## Overview

### Problem Statement

We need continuous integration, to test that changes don't regress functionality as well as to eventually publish binaries to various places (pypi, homebrew)

### Context (as needed)

### Goals

- Build and test rust version of idr-tools automatically
- Enable building python wheels of idr-tools automatically

### Non-Goals

- Publishing source/docs to crates.io
- Publishing python wheels (later) to pypi
- Uploading build artifacts for manual inspection (CI only validates build/test)

### Proposed Solution

Use GitHub Actions to run a small, explicit CI matrix that builds the Rust binaries and Python wheels and runs tests on the three primary platforms (Linux, macOS, Windows). Keep the workflow intentionally minimal: a single build/test job per OS and a small set of architectures that match our release targets, with no publishing steps.
Because the Python package ships only a binary, CI will target a single stable Python version (3.11) rather than a full Python-version matrix.
As a basic packaging check, CI will install the built wheel and run `idr --help`.

## Future plans (as needed)

- Publish to pypi, crates.io, homebrew etc. via some kind of CI

## Other reading (as needed)

- [2026-01-12: Add python packaging](./202601120049-add-python-packaging.md)

## Implementation (ephemeral)

- Add `.github/workflows/ci.yml` with a matrix for `ubuntu-latest`, `macos-latest`, and `windows-latest`.
- Use a Rust toolchain step and run `cargo test` (and `cargo build --release` if needed for binary artifacts).
- Use `actions/setup-python` (pin a supported Python, e.g. 3.11) and `maturin-action` to build wheels per OS (matching the existing `pyproject.toml`).
- Install the built wheel, run `idr --help`, and generate an IDR in a temp directory to validate the packaged binary works end-to-end.
- Serialize env-var tests (e.g., `serial_test`) to avoid CI flakiness from parallel test execution.

Status:
- ✅ Added `.github/workflows/ci.yml` with OS matrix, Rust tests, and maturin wheel builds.
- ✅ Pinned CI to Python 3.11 for binary-only packaging.
- ✅ Added a smoke test that installs the wheel and verifies the generated IDR has a title and `Owner:` line.
- ✅ Serialized env-var tests using `serial_test` to avoid Windows CI flakiness.
