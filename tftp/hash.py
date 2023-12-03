import sys
import base64
from Crypto.Hash import SHA256
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.serialization import (
    load_pem_private_key,
    load_pem_public_key,
)

# 4KiB blocks
BUF_SIZE = 4096
sha256_hash = SHA256.new()

if __name__ == "__main__":
    with open("private_key.pem", "rb") as file:
        private_key = load_pem_private_key(file.read(), password=None)
    privatePem = private_key.private_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PrivateFormat.PKCS8,
        encryption_algorithm=serialization.NoEncryption(),
    )
    print("private key:\n", privatePem.decode())

    with open("public_key.pem", "rb") as file:
        public_key = load_pem_public_key(file.read())
    publicPem = public_key.public_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PublicFormat.SubjectPublicKeyInfo,
    )
    print("Public key:\n", publicPem.decode())

    byte_count = 0
    with open(sys.argv[1], "rb") as binary:
        while True:
            data = binary.read(BUF_SIZE)
            if not data:
                break
            byte_count += len(data)
            sha256_hash.update(data)

    print(f"Processed {byte_count} bytes\n")
    hashed_data = sha256_hash.digest()
    print(f"Digest: [{', '.join(hex(b) for b in hashed_data)}]")
    print(f"Length: {len(hashed_data)}\n")

    signature = private_key.sign(hashed_data)
    print(f"Signature: {signature}\n")

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
