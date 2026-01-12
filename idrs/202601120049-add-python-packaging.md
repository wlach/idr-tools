# 2026-01-12: Add python packaging

Owner: Will Lachance <wlach@protonmail.com>

## Overview

### Problem Statement

Enable easy installation and use of the `idr` tool from Python-based projects.

### Context (as needed)

Python developers should be able to add `idr-tools` as a dependency and immediately use the `idr` command without needing to separately install Rust or the binary.

### Goals

- Allow running `idr` command via `uv run idr new "Title"` after `uv add idr-tools`
- Work seamlessly in any Python project without requiring separate binary installation

### Non-goals

- Publishing to PyPI (we'll deal with that later)
- Providing a Python library API for programmatically creating IDRs (YAGNI - can add later if needed)

### Proposed Solution

Use maturin with `bindings = "bin"` to package the Rust `idr` binary as a Python package, similar to how `uv` is distributed. This provides a simple, maintainable solution focused on the CLI use case.

**User experience:**

```sh
uv add idr-tools
uv run idr new "My New Feature"
```

**Not included:** Python library API for programmatic IDR creation (can be added later if a real use case emerges).

## Detailed Design (as needed)

### Distribution Model

Follow the `uv` approach: distribute a pure Rust binary via Python packaging infrastructure using maturin with `bindings = "bin"`. This avoids PyO3 complexity while achieving the goal of making `idr` easily available to Python projects.

## Other reading (as needed)

See [202601031709-init.md](./202601031709-init.md) for details on the original format

## Implementation (ephemeral)

### Initial Exploration: PyO3 Approach (Abandoned)

Initially explored creating Python bindings using PyO3 to provide both a CLI binary and a Python library API. This involved:

- Refactoring Rust code into `src/lib.rs` with `pub fn create_idr()`
- Creating `src/idr.rs` by consolidating `utils.rs` logic
- Adding PyO3 bindings with conditional compilation
- Creating `python/idr_tools/__init__.py` wrapper
- Building complex `build.py` script to bundle the binary
- Setting up `_run_binary()` to locate and execute the binary

**Why abandoned:** Overly complex for the actual requirement. YAGNI - no real use case for programmatic Python API.

### Final Implementation: Binary-Only Distribution

**Followed the `uv` model**: Use maturin with `bindings = "bin"` to distribute just the Rust binary via Python packaging.

**Changes made:**

1. **Updated `Cargo.toml`**

   - Removed `[lib]` section and `crate-type`
   - Removed `pyo3` dependency and `python` feature
   - Kept only the `[[bin]]` section for the `idr` binary
   - Kept `[profile.release]` with size optimizations (`strip = true`, `opt-level = "z"`, `lto = true`, `codegen-units = 1`)

2. **Simplified `pyproject.toml`**

   - Changed from `bindings = "pyo3"` to `bindings = "bin"`
   - Removed `python-source`, `module-name`, and `features` settings
   - Removed `[project.scripts]` entry (maturin handles this automatically)
   - Removed `strip = true` (redundant - handled by Cargo release profile)

3. **Cleaned up source code**

   - Removed all PyO3 code from `src/lib.rs` (kept only public Rust API)
   - Deleted `python/` directory entirely
   - Deleted `build.py`, `MANIFEST.in`, and wrapper scripts
   - Deleted `tests/test_idr_tools.py`

4. **Kept from refactoring work**
   - `src/lib.rs` with `pub fn create_idr()` for potential Rust library consumers
   - `src/idr.rs` with core IDR creation logic (consolidated from `utils.rs`)
   - `src/main.rs` as thin CLI wrapper calling the library
   - All 19 Rust unit tests

### Testing & Validation

1. **Built the wheel**

   ```bash
   maturin build --release
   ```

   - Produced `py3-none-manylinux` wheel (platform-agnostic since it's just a binary)
   - Verified binary included at `idr_tools-0.1.0.data/scripts/idr` (~6MB)

2. **Tested installation in separate project**

   - Created test project with `uv init`
   - Ran `uv add idr-tools` (from local wheel)
   - Verified `uv run idr new "Test Title"` works
   - Tested `--no-comments` flag
   - All functionality working as expected

3. **Results**
   - ✅ Binary correctly bundled and installed to virtual environment
   - ✅ `uv run idr` invokes the Rust binary transparently
   - ✅ No Python wrapper code needed - maturin handles everything
   - ✅ Simpler than PyO3 approach (removed ~500 lines of wrapper code)

### Outstanding Work

1. **Update README.md**

   - Add Python packaging installation instructions
   - Document `uv add idr-tools` workflow
   - Keep existing Rust and binary usage documentation

2. **Publish to PyPI**

   - Build wheels for multiple platforms (Linux, macOS, Windows)
   - Set up CI/CD for automated builds (GitHub Actions with maturin-action)
   - Test installation from PyPI: `uv add idr-tools`

3. **Multi-platform support**
   - Currently built for Linux (`manylinux`)
   - Need to add macOS and Windows builds
   - Consider platform-specific binary naming if needed

### Key Files Summary

**Files kept from refactoring:**

- `src/lib.rs` - Public Rust library API (no PyO3 code)
- `src/idr.rs` - Core IDR creation logic (consolidated from utils.rs)
- `src/main.rs` - Thin CLI wrapper calling library
- `src/git.rs` - Git identity resolution (unchanged)

**Files added:**

- `pyproject.toml` - Maturin configuration with `bindings = "bin"`

**Files removed in simplification:**

- `src/utils.rs` - Consolidated into `src/idr.rs`
- `python/` directory - Not needed for binary-only distribution
- `build.py` - Not needed (maturin handles everything)
- `MANIFEST.in` - Not needed
- `tests/test_idr_tools.py` - Python tests removed (Rust tests remain)

### Implementation Progress

**Refactoring Phase (Completed)**

Successfully refactored the Rust codebase into library and binary components:

- Created `src/idr.rs` with core IDR creation logic (filename generation, path resolution, HTML comment stripping, template rendering)
- Created `src/lib.rs` with public `create_idr()` API (for Rust consumers)
- Simplified `src/main.rs` to thin CLI wrapper (42 lines) that calls library function
- All 19 Rust unit tests pass (`cargo test`)
- Binary functionality verified: creates IDRs correctly with and without comments

**PyO3 Exploration Phase (Completed then Reverted)**

Built full PyO3 bindings with Python wrapper:

- Added PyO3 dependencies and feature flags to `Cargo.toml`
- Created Python module in `python/idr_tools/__init__.py`
- Built `build.py` script to bundle binary with Python package
- Created 10 comprehensive pytest test cases
- All tests passed successfully

**Why reverted:**

- YAGNI: No actual use case for programmatic Python API
- Overly complex: ~500 lines of wrapper code for what is essentially a CLI tool
- The `uv` model is simpler: just distribute the binary

**Final Simplification (Completed)**

Switched to binary-only distribution following the `uv` approach:

- Changed `pyproject.toml`: `bindings = "pyo3"` → `bindings = "bin"`
- Removed all PyO3 code from `src/lib.rs` (kept public Rust API)
- Deleted `python/` directory, `build.py`, `MANIFEST.in`
- Removed `pyo3` from `Cargo.toml` and `[lib]` section
- Maturin now builds and installs just the binary

**Result:**

- Wheel contains binary at `.data/scripts/idr` (~6MB)
- `uv add idr-tools` → `uv run idr new "Title"` works as intended
- No Python wrapper needed - maturin handles everything
- Simpler, smaller, easier to maintain

**Validation:**

- Built release wheel with `maturin build --release`
- Tested installation in separate project
- Verified `uv run idr` works with all flags
- Binary size: ~6MB (gix is the largest contributor at ~5MB)
