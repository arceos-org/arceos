#!/usr/bin/env bash

ELF=$1
OBJCOPY=$2

if [ -z "$ELF" ] || [ -z "$OBJCOPY" ]; then
    echo "Usage: $0 <elf-file> <objcopy-command>"
    exit 1
fi

if [ ! -f "$ELF" ]; then
    echo "Error: ELF file '$ELF' does not exist."
    exit 1
fi

SECTIONS=(
    debug_abbrev
    debug_addr
    debug_aranges
    debug_info
    debug_line
    debug_line_str
    debug_ranges
    debug_rnglists
    debug_str
    debug_str_offsets
)

for section in "${SECTIONS[@]}"; do
    $OBJCOPY $ELF --dump-section .$section=$section.bin 2> /dev/null || touch $section.bin &
done
wait
$OBJCOPY $ELF --strip-debug

cmd=($OBJCOPY $ELF)
for section in "${SECTIONS[@]}"; do
    cmd+=(--update-section $section=$section.bin)
    cmd+=(--rename-section $section=.$section)
done
${cmd[@]}

for section in "${SECTIONS[@]}"; do
    rm -f $section.bin
done
