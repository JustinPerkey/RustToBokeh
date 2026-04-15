#!/usr/bin/env bash
# fetch_bokeh_js.sh — Download Bokeh 3.9.0 JS/widgets files for offline
# (BokehResources::Inline) rendering without requiring the full setup_vendor.sh.
#
# Tries three strategies in order:
#   1. Copy from an already-installed system/venv Python Bokeh package.
#   2. Use npm / npx to pull @bokeh/bokehjs@3.9.0.
#   3. Point to vendor/python if setup_vendor.sh has already run.
#
# Usage:  bash scripts/fetch_bokeh_js.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DEST="$PROJECT_DIR/vendor/bokeh"
VERSION="3.9.0"

JS_DEST="$DEST/bokeh-${VERSION}.min.js"
WIDGETS_DEST="$DEST/bokeh-widgets-${VERSION}.min.js"

if [ -f "$JS_DEST" ] && [ -f "$WIDGETS_DEST" ]; then
    echo "vendor/bokeh/ already present — nothing to do."
    exit 0
fi

mkdir -p "$DEST"

copy_from_python() {
    local python_exe="$1"
    local static
    static="$("$python_exe" -c "
import importlib.util, os, sys
spec = importlib.util.find_spec('bokeh')
if spec is None: sys.exit(1)
pkg_dir = os.path.dirname(spec.origin)
static = os.path.join(pkg_dir, 'server', 'static', 'js')
if not os.path.isdir(static): sys.exit(1)
print(static)
" 2>/dev/null)" || return 1
    [ -f "$static/bokeh.min.js" ] || return 1
    cp "$static/bokeh.min.js" "$JS_DEST"
    cp "$static/bokeh-widgets.min.js" "$WIDGETS_DEST"
    echo "Copied Bokeh $VERSION JS from Python package ($python_exe)."
    return 0
}

# ── Strategy 1: system/venv Python ──────────────────────────────────────────
for py in python3 python; do
    if command -v "$py" &>/dev/null; then
        if copy_from_python "$(command -v "$py")"; then
            exit 0
        fi
    fi
done

# ── Strategy 2: vendored Python (if setup_vendor.sh already ran) ─────────────
VENDOR_PY="$PROJECT_DIR/vendor/python/bin/python3"
if [ -f "$VENDOR_PY" ]; then
    if copy_from_python "$VENDOR_PY"; then
        exit 0
    fi
fi

# ── Strategy 3: npm / npx ────────────────────────────────────────────────────
if command -v npm &>/dev/null || command -v npx &>/dev/null; then
    echo "Trying npm install @bokeh/bokehjs@${VERSION} ..."
    TMP_DIR="$(mktemp -d)"
    trap 'rm -rf "$TMP_DIR"' EXIT
    cd "$TMP_DIR"
    if npm install "@bokeh/bokehjs@${VERSION}" --prefer-offline 2>/dev/null; then
        JS_SRC="$TMP_DIR/node_modules/@bokeh/bokehjs/build/js/bokeh.min.js"
        W_SRC="$TMP_DIR/node_modules/@bokeh/bokehjs/build/js/bokeh-widgets.min.js"
        if [ -f "$JS_SRC" ] && [ -f "$W_SRC" ]; then
            cp "$JS_SRC" "$JS_DEST"
            cp "$W_SRC"  "$WIDGETS_DEST"
            echo "Copied Bokeh $VERSION JS from @bokeh/bokehjs npm package."
            cd "$PROJECT_DIR"
            exit 0
        fi
    fi
    cd "$PROJECT_DIR"
fi

echo "ERROR: Could not obtain Bokeh $VERSION JS files." >&2
echo "Options:" >&2
echo "  1. Install Python Bokeh:  pip install bokeh==${VERSION}   then re-run this script." >&2
echo "  2. Run the full setup:    bash scripts/setup_vendor.sh" >&2
echo "  3. Use CDN mode:          change BokehResources::Inline to Cdn in main.rs" >&2
exit 1
