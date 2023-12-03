with open("public_key.raw", "rb") as f:
    print("Public key:", end=" ")
    for item in f.read():
        print(item, end=" ")
    print()

with open("private_key.raw", "rb") as f:
    print("Private key:", end=" ")
    for item in f.read():
        print(item, end=" ")
    print()
