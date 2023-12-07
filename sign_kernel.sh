#!/bin/bash
(cd tftp &&
python3 generate_ed25519_keys.py && \
gzip --decompress --keep -f Image.gz && \
python3 hash.py Image && \
gzip -f Image_signed && rm Image)

(cd tftp &&
gzip --decompress --keep -f Image-vf2.gz && \
python3 hash.py Image-vf2 && \
mv Image-vf2_signed Image_vf2_signed && \
gzip -f Image_vf2_signed && rm Image-vf2)
