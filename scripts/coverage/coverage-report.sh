#!/bin/bash
# Extract axtest coverage raw data from disk image and generate HTML report.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

DISK_IMG="${1:-${PROJECT_ROOT}/disk.img}"
APP_ELF="${2:-${PROJECT_ROOT}/examples/axtest-runner/axtest-runner_aarch64-qemu-virt.elf}"
OUTPUT_DIR="${3:-${PROJECT_ROOT}/coverage_report}"

RAW_NAME="axtest_cov.profraw"
RAW_PATH="${OUTPUT_DIR}/${RAW_NAME}"
PROFDATA_PATH="${OUTPUT_DIR}/coverage.profdata"
SUMMARY_PATH="${OUTPUT_DIR}/coverage_summary.txt"
COV_LOG="${OUTPUT_DIR}/llvm-cov.log"
EXPORT_JSON="${OUTPUT_DIR}/coverage.json"

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

find_llvm_tool() {
    local tool="$1"

    if command -v "$tool" >/dev/null 2>&1; then
        command -v "$tool"
        return 0
    fi

    local sysroot
    local host
    sysroot="$(rustc --print sysroot)"
    host="$(rustc -vV | sed -n 's/host: //p')"

    local candidates=(
        "$sysroot/lib/rustlib/$host/bin/$tool"
        "$sysroot/lib/rustlib/aarch64-apple-darwin/bin/$tool"
        "$sysroot/lib/rustlib/x86_64-apple-darwin/bin/$tool"
        "$sysroot/lib/rustlib/aarch64-unknown-linux-gnu/bin/$tool"
        "$sysroot/lib/rustlib/x86_64-unknown-linux-gnu/bin/$tool"
    )

    local candidate
    for candidate in "${candidates[@]}"; do
        if [ -f "$candidate" ]; then
            echo "$candidate"
            return 0
        fi
    done

    return 1
}

echo -e "${BLUE}=== Extract And Generate Coverage ===${NC}"
echo "disk image: $DISK_IMG"
echo "app elf:    $APP_ELF"
echo "output dir: $OUTPUT_DIR"

if [ ! -f "$DISK_IMG" ]; then
    echo -e "${RED}Error: disk image not found: $DISK_IMG${NC}"
    exit 1
fi

if [ ! -f "$APP_ELF" ]; then
    echo -e "${RED}Error: app ELF not found: $APP_ELF${NC}"
    exit 1
fi

if ! command -v mcopy >/dev/null 2>&1; then
    echo -e "${RED}Error: mcopy not found. Install mtools first.${NC}"
    exit 1
fi

mkdir -p "$OUTPUT_DIR"
rm -f "$RAW_PATH" "$PROFDATA_PATH" "$SUMMARY_PATH" "$COV_LOG" "$EXPORT_JSON"

echo -e "\n${BLUE}[1/3] Extracting ${RAW_NAME} from disk image...${NC}"
mcopy -i "$DISK_IMG" "::/${RAW_NAME}" "$RAW_PATH"

LLVM_PROFDATA="$(find_llvm_tool llvm-profdata || true)"
LLVM_COV="$(find_llvm_tool llvm-cov || true)"

if [ -z "$LLVM_PROFDATA" ] || [ -z "$LLVM_COV" ]; then
    echo -e "${YELLOW}llvm tools not found, trying to install llvm-tools-preview...${NC}"
    rustup component add llvm-tools-preview >/dev/null
    LLVM_PROFDATA="$(find_llvm_tool llvm-profdata || true)"
    LLVM_COV="$(find_llvm_tool llvm-cov || true)"
fi

if [ -z "$LLVM_PROFDATA" ] || [ -z "$LLVM_COV" ]; then
    echo -e "${RED}Error: cannot find llvm-profdata/llvm-cov${NC}"
    exit 1
fi

echo -e "\n${BLUE}[2/3] Merging profraw -> profdata...${NC}"
"$LLVM_PROFDATA" merge -sparse "$RAW_PATH" -o "$PROFDATA_PATH"

echo -e "\n${BLUE}[3/3] Generating HTML report...${NC}"
"$LLVM_COV" show \
    --instr-profile="$PROFDATA_PATH" \
    --format=html \
    --output-dir="$OUTPUT_DIR" \
    "$APP_ELF" \
    > "$COV_LOG" 2>&1

"$LLVM_COV" report \
    --instr-profile="$PROFDATA_PATH" \
    "$APP_ELF" \
    | tee "$SUMMARY_PATH"

"$LLVM_COV" export \
    --instr-profile="$PROFDATA_PATH" \
    --format=json \
    "$APP_ELF" \
    > "$EXPORT_JSON" 2>/dev/null || true

echo -e "\n${GREEN}Done.${NC}"
echo "raw:      $RAW_PATH"
echo "profdata: $PROFDATA_PATH"
echo "html:     $OUTPUT_DIR/index.html"
