# stickers

An attempt at archiving stickers.

## Adding new stickers

1. Add a photo to `public/stickers/raw`.
2. Run `./kill_exif.sh` which will clean up the new raw photo.
3. Run `./extract_stickers.sh` to verify that everything works correctly. Go to
   `public/stickers/iamges` and confirm that the photos were processed
   correctly.
4. Add the sticker to `public/stickers/stickers.json`.
