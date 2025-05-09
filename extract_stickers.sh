#!/bin/bash
set -e

target_dir="public/stickers/images";
rm -rf ./$target_dir/* # :worry:
target_dir=$(realpath $target_dir);

source_dir="public/stickers/raw";
source_dir=$(realpath $source_dir);

echo "$source $target";
(cd extractor-rust && cargo run --release -- directory "$source_dir" "$target_dir");
