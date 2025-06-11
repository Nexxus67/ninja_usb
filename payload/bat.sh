#!/bin/bash
set -e

IMG=fs_image_32kb.img
MOUNTDIR=usb_temp

# Clean old image
rm -f "$IMG"
rm -rf "$MOUNTDIR"

# Create empty FAT16 image (64 sectors of 512B = 32KB)
dd if=/dev/zero of=$IMG bs=512 count=64
mkfs.fat -F 16 $IMG

# Mount it using mtools (no root needed)
mkdir "$MOUNTDIR"
mcopy -i $IMG start.bat ::
mcopy -i $IMG payload.dll ::
mcopy -i $IMG Loader.exe ::

# Verify
mdir -i $IMG ::
echo "[OK] Image $IMG built with start.bat, payload.dll, Loader.exe"

