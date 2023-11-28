import sys
from Crypto.PublicKey import RSA
from Crypto.Signature import PKCS1_PSS
from Crypto.Hash import SHA256

# 4KiB blocks
BUF_SIZE = 4096
sha256_hash = SHA256.new()

if __name__ == "__main__":
    with open("private_key.pem", "r") as file:
        private_key = RSA.importKey(file.read())

    with open("public_key.pem", "r") as file:
        public_key = RSA.importKey(file.read())

    with open(sys.argv[1], "rb") as binary:
        while True:
            data = binary.read(BUF_SIZE)
            if not data:
                break
            sha256_hash.update(data)

    digest = sha256_hash
    print(
        f"Digest: {digest.hexdigest()} with length {len(bytes.fromhex(digest.hexdigest()))}\n"
    )

    print(public_key.export_key().decode() + "\n")
    print(private_key.export_key().decode() + "\n")

    signature = PKCS1_PSS.new(private_key).sign(digest)
    print(f"Signature: {signature}")

    try:
        PKCS1_PSS.new(public_key).verify(digest, signature)
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
