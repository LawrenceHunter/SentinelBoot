#!/bin/bash

python3 generate_ed25519_keys.py && \
gzip --decompress --keep Image.gz && \
python3 hash.py Image && \
gzip Image_signed && rm Image
