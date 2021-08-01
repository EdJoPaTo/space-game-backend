#!/usr/bin/env bash
set -e

rm ./*.ts 2> /dev/null || true
cargo test

targetfile='typescript-typings/generated.ts'
cat typescript-typings/manual.ts > $targetfile

for file in ./*.ts ; do
    printf "\n" >> $targetfile
    # 1: bla | null -> bla | undefined
    # 2: remove imports as everything is in the same file
    # 3: remove empty lines
    sed -E \
        -e "s#: (\S+) \| null;#\?: \1;#g" \
        -e "s#^import.+##g" \
        -e "/^$/d" \
        "$file" >> $targetfile
    rm "$file"
done

# Add final newline
printf "\n" >> $targetfile
