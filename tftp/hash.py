import sys
import base64
from Crypto.Hash import SHA256
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import (
    Ed25519PrivateKey,
    Ed25519PublicKey,
)

# 4KiB blocks
BUF_SIZE = 4096
sha256_hash = SHA256.new()

if __name__ == "__main__":
    with open("private_key.pem", "rb") as file:
        private_key = Ed25519PrivateKey.from_private_bytes(file.read())
    privatePem = private_key.private_bytes(
        encoding=serialization.Encoding.Raw,
        format=serialization.PrivateFormat.Raw,
        encryption_algorithm=serialization.NoEncryption(),
    )
    print("Private key:", end=" ")
    for item in privatePem:
        print(hex(item)[2:], end=" ")
    print()

    with open("public_key.pem", "rb") as file:
        public_key = Ed25519PublicKey.from_public_bytes(file.read())
    publicPem = public_key.public_bytes(
        encoding=serialization.Encoding.Raw,
        format=serialization.PublicFormat.Raw,
    )
    print("Public key:", end=" ")
    for item in publicPem:
        print(hex(item)[2:], end=" ")
    print()

    byte_count = 0
    with open(sys.argv[1], "rb") as binary:
        while True:
            data = binary.read(BUF_SIZE)
            if not data:
                break
            byte_count += len(data)
            sha256_hash.update(data)

    print(f"\nProcessed {byte_count} bytes\n")
    hashed_data = sha256_hash.digest()
    print("Hash:", end=" ")
    for item in hashed_data:
        print(hex(item)[2:], end=" ")
    print()
    print(f"Length: {len(hashed_data)}\n")

    signature = private_key.sign(hashed_data)
    print("Signature:", end=" ")
    for item in signature:
        print(hex(item)[2:], end=" ")
    print()
    print()

    try:
        public_key.verify(signature, hashed_data)
        print("Signature is valid.")
    except (ValueError, TypeError):
        print("Signature is invalid or key mismatch.")

    with open(sys.argv[1], "rb") as old, open(
        sys.argv[1] + "_signed", "wb"
    ) as new:
        new.write(signature)
        new.write(bytearray(256 - len(signature)))
        for chunk in iter(lambda: old.read(1024), b""):
            new.write(chunk)