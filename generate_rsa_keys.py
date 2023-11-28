from Crypto.PublicKey import RSA

key_pair = RSA.generate(1024)
public_key = key_pair.publickey()

if __name__ == "__main__":
    with open("private_key.pem", "wb") as file:
        file.write(key_pair.exportKey("PEM"))

    with open("public_key.pem", "wb") as file:
        file.write(public_key.exportKey("PEM"))
