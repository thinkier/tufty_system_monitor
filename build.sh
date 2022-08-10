#!/bin/bash
OUT_DIR="../pico-project/artefacts/"

cmake ../pico-project || exit 1
make || exit 1

mkdir $OUT_DIR >/dev/null 2>&1
FORMATS=("bin" "dis" "elf" "elf.map" "hex" "uf2")
for FORMAT in "${FORMATS[@]}"; do
  cp *.$FORMAT $OUT_DIR
done