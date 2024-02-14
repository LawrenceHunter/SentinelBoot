ops = ["vsha2ms.vv", "vsha2cl.vv", "vsha2ch.vv", "vrev8.v"]


def register_map(reg_mnemonic):
    if reg_mnemonic[0] == "x":
        return int(reg_mnemonic[1:])
    elif reg_mnemonic[0] == "v":
        return int(reg_mnemonic[1:])
    elif reg_mnemonic == "zero":
        return 0
    elif reg_mnemonic == "ra":
        return 1
    elif reg_mnemonic == "sp":
        return 2
    elif reg_mnemonic == "gp":
        return 3
    elif reg_mnemonic == "tp":
        return 4
    elif reg_mnemonic == "t0":
        return 5
    elif reg_mnemonic == "t1":
        return 6
    elif reg_mnemonic == "t2":
        return 7
    elif reg_mnemonic == "s0":
        return 8
    elif reg_mnemonic == "s1":
        return 9
    elif reg_mnemonic == "a0":
        return 10
    elif reg_mnemonic == "a1":
        return 11
    elif reg_mnemonic == "a2":
        return 12
    elif reg_mnemonic == "a3":
        return 13
    elif reg_mnemonic == "a4":
        return 14
    elif reg_mnemonic == "a5":
        return 15
    elif reg_mnemonic == "a6":
        return 16
    elif reg_mnemonic == "a7":
        return 17
    elif reg_mnemonic == "s2":
        return 18
    elif reg_mnemonic == "s3":
        return 19
    elif reg_mnemonic == "s4":
        return 20
    elif reg_mnemonic == "s5":
        return 21
    elif reg_mnemonic == "s6":
        return 22
    elif reg_mnemonic == "s7":
        return 23
    elif reg_mnemonic == "s8":
        return 24
    elif reg_mnemonic == "s9":
        return 25
    elif reg_mnemonic == "s10":
        return 26
    elif reg_mnemonic == "s11":
        return 27
    elif reg_mnemonic == "t3":
        return 28
    elif reg_mnemonic == "t4":
        return 29
    elif reg_mnemonic == "t5":
        return 30
    elif reg_mnemonic == "t6":
        return 31
    else:
        raise ValueError("Invalid register mnemonic")


with open("./verification.rs", "r") as f:
    instructions = []
    for line in f.readlines():
        line = line.strip()
        if line.startswith("//"):
            line = line.replace(",", "").split(" ")[1:]
            if line[0] in ops:
                instructions.append(line)

    processed_instructions = []
    for instruction in instructions:
        if instruction[0] == "vsha2ms.vv":
            bits = ["101101", "1", "xxxxx", "xxxxx", "010", "xxxxx", "1110111"]
            bits[2] = format(register_map(instruction[2]), "05b")
            bits[3] = format(register_map(instruction[3]), "05b")
            bits[5] = format(register_map(instruction[1]), "05b")
            processed_instructions.append(bits)
        elif instruction[0] == "vsha2cl.vv":
            bits = ["101111", "1", "xxxxx", "xxxxx", "010", "xxxxx", "1110111"]
            bits[2] = format(register_map(instruction[2]), "05b")
            bits[3] = format(register_map(instruction[3]), "05b")
            bits[5] = format(register_map(instruction[1]), "05b")
            processed_instructions.append(bits)
        elif instruction[0] == "vsha2ch.vv":
            bits = ["101110", "1", "xxxxx", "xxxxx", "010", "xxxxx", "1110111"]
            bits[2] = format(register_map(instruction[2]), "05b")
            bits[3] = format(register_map(instruction[3]), "05b")
            bits[5] = format(register_map(instruction[1]), "05b")
            processed_instructions.append(bits)
        elif instruction[0] == "vrev8.v":
            bits = ["010010", "x", "xxxxx", "01001", "010", "xxxxx", "1010111"]
            bits[1] = "1"
            bits[2] = format(register_map(instruction[2]), "05b")
            bits[5] = format(register_map(instruction[1]), "05b")
            processed_instructions.append(bits)

    with open("output", "w") as f:
        for i, instruction in enumerate(processed_instructions):
            f.write(
                ".word "
                + hex(int("0b" + "".join(instruction), 2))
                + ", // "
                + " ".join(instructions[i])
                + "\n"
            )
