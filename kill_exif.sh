#!/bin/bash
set -e

exiv2 -M"del Exif.Image.Orientation" ./public/stickers/raw/*
