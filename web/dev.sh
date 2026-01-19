#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
WASM_CRATE="$ROOT_DIR/crates/wc-wasm"
WASM_PKG="$SCRIPT_DIR/wasm-pkg"

echo -e "${BLUE}=== WC Predictor Dev Server ===${NC}"

# 1. Kill any existing vite dev server
echo -e "${YELLOW}Checking for existing dev server...${NC}"
if pgrep -f "vite" > /dev/null 2>&1; then
    echo -e "${YELLOW}Killing existing vite process...${NC}"
    pkill -f "vite" || true
    sleep 1
fi
echo -e "${GREEN}✓ No conflicting processes${NC}"

# 2. Check if WASM needs to be rebuilt
echo -e "${YELLOW}Checking WASM build status...${NC}"

WASM_OUTPUT="$WASM_PKG/wc_wasm_bg.wasm"
NEEDS_WASM_BUILD=false

if [ ! -f "$WASM_OUTPUT" ]; then
    echo -e "${YELLOW}WASM output not found, build required${NC}"
    NEEDS_WASM_BUILD=true
else
    # Find the most recently modified Rust source file
    NEWEST_RS=$(find "$ROOT_DIR/crates" -name "*.rs" -type f -newer "$WASM_OUTPUT" 2>/dev/null | head -1)
    NEWEST_TOML=$(find "$ROOT_DIR/crates" -name "Cargo.toml" -type f -newer "$WASM_OUTPUT" 2>/dev/null | head -1)

    if [ -n "$NEWEST_RS" ] || [ -n "$NEWEST_TOML" ]; then
        echo -e "${YELLOW}Rust source files changed since last WASM build${NC}"
        NEEDS_WASM_BUILD=true
    fi
fi

if [ "$NEEDS_WASM_BUILD" = true ]; then
    echo -e "${BLUE}Building WASM module...${NC}"

    # Check if wasm-pack is installed
    if ! command -v wasm-pack &> /dev/null; then
        echo -e "${RED}Error: wasm-pack not found. Install with: cargo install wasm-pack${NC}"
        exit 1
    fi

    cd "$WASM_CRATE"
    wasm-pack build --target web --release --out-dir "$WASM_PKG"
    cd "$SCRIPT_DIR"
    echo -e "${GREEN}✓ WASM build complete${NC}"
else
    echo -e "${GREEN}✓ WASM is up to date${NC}"
fi

# 3. Check if npm install is needed
echo -e "${YELLOW}Checking npm dependencies...${NC}"

cd "$SCRIPT_DIR"

if [ ! -d "node_modules" ]; then
    echo -e "${YELLOW}node_modules not found, running npm install...${NC}"
    npm install
    echo -e "${GREEN}✓ npm install complete${NC}"
elif [ "package.json" -nt "node_modules" ] || [ "package-lock.json" -nt "node_modules" ]; then
    echo -e "${YELLOW}package.json changed, running npm install...${NC}"
    npm install
    echo -e "${GREEN}✓ npm install complete${NC}"
else
    echo -e "${GREEN}✓ npm dependencies up to date${NC}"
fi

# 4. Run lint check (optional, quick)
echo -e "${YELLOW}Running lint check...${NC}"
if npm run lint 2>&1 | grep -q "error"; then
    echo -e "${RED}Lint errors found. Fix them before continuing.${NC}"
    npm run lint
    exit 1
fi
echo -e "${GREEN}✓ Lint check passed${NC}"

# 5. Start the dev server
echo -e "${BLUE}Starting dev server...${NC}"
echo -e "${GREEN}Server will be available at: http://localhost:5173/wc_predictor/${NC}"
echo ""

npm run dev
