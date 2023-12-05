import sys

with open(sys.argv[1], "rb") as old, open(
    sys.argv[1] + "_mangled", "wb"
) as new:
    for chunk in iter(lambda: old.read(1024), b""):
        new.write(chunk)
    new.write(bytearray(256))
