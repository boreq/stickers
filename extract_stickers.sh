#!/bin/bash
set -e

for filename in ./public/stickers/raw/*; do
    source=$filename;
    target="./public/stickers/images/$(basename ${filename%.jpg}).png";

    source=$(realpath $source);
    target=$(realpath $target);

    echo "$source $target";
    (cd extractor-rust && cargo run --release "$source" "$target");
done
