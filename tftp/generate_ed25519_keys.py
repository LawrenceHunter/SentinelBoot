from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

private_key = Ed25519PrivateKey.generate()
public_key = private_key.public_key()

if __name__ == "__main__":
    with open("public_key.pem", "wb") as file:
        file.write(public_key.public_bytes_raw())

    with open("private_key.pem", "wb") as file:
        file.write(private_key.private_bytes_raw())
