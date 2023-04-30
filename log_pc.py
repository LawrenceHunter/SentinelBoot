import matplotlib.pyplot as plt

addresses = []

with open("gdb.txt", "r") as f:
    for line in f.readlines():
        if "=>" in line:
            addresses.append(line.split(" ")[1].split(":")[0])

plt.figure(figsize=(15, 10))
plt.plot(addresses)
plt.xlabel("Time (cycle)")
plt.ylabel("Instruction address")
plt.title("PC Path")
plt.yticks(["0x80000024", "0x8000050e", "0x80000a9e", "0x80001582"])
plt.savefig("pc_flow.png")
