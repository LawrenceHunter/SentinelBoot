from cryptography.hazmat.primitives.asymmetric.ed25519 import (
    Ed25519PrivateKey,
    Ed25519PublicKey,
)

from cryptography.hazmat.primitives.serialization import (
    Encoding,
    PrivateFormat,
    PublicFormat,
)

from cryptography.hazmat.primitives import serialization

private_key = Ed25519PrivateKey.generate()
public_key = private_key.public_key()

if __name__ == "__main__":
    with open("private_key.pem", "wb") as file:
        file.write(
            private_key.private_bytes(
                encoding=Encoding.PEM,
                format=PrivateFormat.PKCS8,
                encryption_algorithm=serialization.NoEncryption(),
            )
        )

    with open("public_key.pem", "wb") as file:
        file.write(
            public_key.public_bytes(
                encoding=Encoding.PEM, format=PublicFormat.SubjectPublicKeyInfo
            )
        )

    with open("public_key.raw", "wb") as file:
        file.write(
            public_key.public_bytes(
                encoding=Encoding.Raw, format=PublicFormat.Raw
            )
        )
