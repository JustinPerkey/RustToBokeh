#!/usr/bin/env bash
# setup_vendor.sh — Download a standalone Python build and install project
# dependencies into it. No system Python installation required.
#
# Usage:  bash scripts/setup_vendor.sh
#
# The script downloads a portable CPython build from
# https://github.com/indygreg/python-build-standalone, extracts it to
# vendor/python/, installs pip packages from requirements.txt, and writes
# .cargo/config.toml so that PyO3 links against the vendored interpreter.

set -euo pipefail

# ── Configuration ────────────────────────────────────────────────────────────

PYTHON_VERSION="3.12.8"
RELEASE_TAG="20250106"
BASE_URL="https://github.com/indygreg/python-build-standalone/releases/download/${RELEASE_TAG}"

# ── Resolve paths ────────────────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
VENDOR_DIR="$PROJECT_DIR/vendor/python"

# ── Detect platform ─────────────────────────────────────────────────────────

detect_platform() {
    local os arch
    case "$(uname -s)" in
        Linux*)           os="unknown-linux-gnu" ;;
        Darwin*)          os="apple-darwin" ;;
        MINGW*|MSYS*|CYGWIN*|*_NT*) os="pc-windows-msvc" ;;
        *)  echo "Error: unsupported OS — $(uname -s)" >&2; exit 1 ;;
    esac
    case "$(uname -m)" in
        x86_64|amd64)     arch="x86_64" ;;
        aarch64|arm64)    arch="aarch64" ;;
        *)  echo "Error: unsupported architecture — $(uname -m)" >&2; exit 1 ;;
    esac
    echo "${arch}-${os}"
}

PLATFORM="$(detect_platform)"

# Windows builds from python-build-standalone include "-shared" in the name.
case "$PLATFORM" in
    *windows*) VARIANT="-shared" ;;
    *)         VARIANT="" ;;
esac
ARCHIVE="cpython-${PYTHON_VERSION}+${RELEASE_TAG}-${PLATFORM}${VARIANT}-install_only.tar.gz"
URL="${BASE_URL}/${ARCHIVE}"

# ── Determine Python executable path ────────────────────────────────────────

case "$PLATFORM" in
    *windows*) PYTHON_EXE="vendor/python/python.exe" ;;
    *)         PYTHON_EXE="vendor/python/bin/python3" ;;
esac

# ── Download & extract ───────────────────────────────────────────────────────

if [ -d "$VENDOR_DIR" ]; then
    echo "vendor/python/ already exists — skipping download."
    echo "  (Delete vendor/python/ and re-run to force a fresh download.)"
else
    echo "Downloading standalone Python ${PYTHON_VERSION} for ${PLATFORM}..."
    echo "  URL: ${URL}"

    TMPFILE="$(mktemp)"
    trap 'rm -f "$TMPFILE"' EXIT

    if command -v curl &>/dev/null; then
        curl -fSL --progress-bar -o "$TMPFILE" "$URL"
    elif command -v wget &>/dev/null; then
        wget -q --show-progress -O "$TMPFILE" "$URL"
    else
        echo "Error: neither curl nor wget found." >&2
        exit 1
    fi

    echo "Extracting to vendor/python/..."
    mkdir -p "$PROJECT_DIR/vendor"
    tar -xzf "$TMPFILE" -C "$PROJECT_DIR/vendor"

    rm -f "$TMPFILE"
    trap - EXIT

    echo "Python extracted to vendor/python/"
fi

# ── Resolve absolute path to interpreter ─────────────────────────────────────

PYTHON_ABS="$PROJECT_DIR/$PYTHON_EXE"

if [ ! -f "$PYTHON_ABS" ]; then
    echo "Error: expected Python at $PYTHON_ABS but it does not exist." >&2
    echo "The archive layout may differ. Check vendor/python/ contents." >&2
    exit 1
fi

echo "Using Python: $PYTHON_ABS"
"$PYTHON_ABS" --version

# ── Install pip packages ────────────────────────────────────────────────────

echo "Bootstrapping pip..."
"$PYTHON_ABS" -m ensurepip --upgrade 2>/dev/null || true

echo "Installing dependencies from requirements.txt..."
"$PYTHON_ABS" -m pip install --upgrade pip setuptools wheel -q
"$PYTHON_ABS" -m pip install -r "$PROJECT_DIR/requirements.txt" -q

echo "Installed packages:"
"$PYTHON_ABS" -m pip list --format=columns

# ── Write .cargo/config.toml ────────────────────────────────────────────────

mkdir -p "$PROJECT_DIR/.cargo"

case "$PLATFORM" in
    *linux*)
        # On Linux we need three things beyond PYO3_PYTHON:
        #   PYTHONHOME    — tells the embedded interpreter where its stdlib lives
        #   LIBRARY_PATH  — biases the *linker* to pick up the vendored libpython
        #                   before any system libpython of the same version
        #   rustflags     — bakes an $ORIGIN-relative rpath into the binary so
        #                   the dynamic linker loads the vendored libpython at
        #                   runtime (takes precedence over /lib/…)
        cat > "$PROJECT_DIR/.cargo/config.toml" <<EOF
[env]
PYO3_PYTHON  = { value = "${PYTHON_EXE}",       relative = true }
PYTHONHOME   = { value = "vendor/python",        relative = true }
LIBRARY_PATH = { value = "vendor/python/lib",    relative = true }

[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "link-arg=-Wl,-rpath,\$ORIGIN/../../vendor/python/lib"]

[target.aarch64-unknown-linux-gnu]
rustflags = ["-C", "link-arg=-Wl,-rpath,\$ORIGIN/../../vendor/python/lib"]
EOF
        ;;
    *darwin*)
        # macOS: PYTHONHOME is sufficient; the dynamic linker respects @rpath
        # set by the vendored Python's own install_name, so no extra flags needed.
        cat > "$PROJECT_DIR/.cargo/config.toml" <<EOF
[env]
PYO3_PYTHON = { value = "${PYTHON_EXE}", relative = true }
PYTHONHOME  = { value = "vendor/python", relative = true }
EOF
        ;;
    *windows*)
        # Windows: PYTHONHOME ensures the embedded interpreter finds its stdlib.
        # build.rs copies the required DLLs to the target directory automatically.
        cat > "$PROJECT_DIR/.cargo/config.toml" <<EOF
[env]
PYO3_PYTHON = { value = "${PYTHON_EXE}", relative = true }
PYTHONHOME  = { value = "vendor/python", relative = true }
EOF
        ;;
esac

echo ""
echo "Wrote .cargo/config.toml with PYO3_PYTHON = ${PYTHON_EXE}"
echo ""
echo "========================================="
echo "  Setup complete!"
echo "  Build with:  cargo build --release"
echo "  Run with:    cargo run --release"
echo "========================================="
