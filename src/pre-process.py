undefined_opcodes = ["vsetvli", "vle64.v", "vsha2ms.vv"]

vle = [0x0205F007, 0x0205F407, 0x0205F807]
vle_counter = 0

vsetvli = 0x018572D7


def register_map(reg_mnemonic):
    if reg_mnemonic[0] == "x":
        return int(reg_mnemonic[1:])
    if reg_mnemonic[0] == "v":
        return int(reg_mnemonic[1:])
    match reg_mnemonic:
        case "zero":
            return 0
        case "ra":
            return 1
        case "sp":
            return 2
        case "gp":
            return 3
        case "tp":
            return 4
        case "t0":
            return 5
        case "t1":
            return 6
        case "t2":
            return 7
        case "s0":
            return 8
        case "s1":
            return 9
        case "a0":
            return 10
        case "a1":
            return 11
        case "a2":
            return 12
        case "a3":
            return 13
        case "a4":
            return 14
        case "a5":
            return 15
        case "a6":
            return 16
        case "a7":
            return 17
        case "s2":
            return 18
        case "s3":
            return 19
        case "s4":
            return 20
        case "s5":
            return 21
        case "s6":
            return 22
        case "s7":
            return 23
        case "s8":
            return 24
        case "s9":
            return 25
        case "s10":
            return 26
        case "s11":
            return 27
        case "t3":
            return 28
        case "t4":
            return 29
        case "t5":
            return 30
        case "t6":
            return 31


def encode_vtypei(e, m, tu, mu):
    sew_encoding = {"e64": "000"}
    lmul_encoding = {"m1": "000"}

    # Convert tail agnostic and mask agnostic settings to binary
    # Assuming True maps to '1' and False to '0'
    tu_bit = "1" if tu == "tu" else "0"
    mu_bit = "1" if mu == "mu" else "0"

    # Combine the parts into the vtypei field
    vtypei = (
        sew_encoding.get(e, "000")
        + lmul_encoding.get(m, "000")
        + tu_bit
        + mu_bit
    )

    # The combined vtypei should be 11 bits long. If it's shorter, pad with zeros.
    vtypei_padded = vtypei.ljust(11, "0")

    return vtypei_padded


if __name__ == "__main__":
    with open("vector_hash.s", "r") as f:
        instructions = []
        for line in f.readlines():
            if "#" in line:
                pass
            else:
                instructions.append(line.strip())

    processed_instructions = []
    for instruction in instructions:
        split_instruction = instruction.replace(",", "").split(" ")
        if split_instruction[0] not in undefined_opcodes:
            processed_instructions.append(instruction)
        else:
            if split_instruction[0] == "vsetvli":
                # "vsetvli": {
                #     "imm": bin(0),
                #     "rs1": bin(0),
                #     "funct3": bin(0),
                #     "opcode": bin(1010111),
                # }
                instruction = (
                    "0"
                    + encode_vtypei(
                        split_instruction[3],
                        split_instruction[4],
                        split_instruction[5],
                        split_instruction[6],
                    )
                    + format(register_map(split_instruction[2]), "05b")
                    + "111"
                    + format(register_map(split_instruction[1]), "05b")
                    + "1010111"
                )
                try:
                    assert format(vsetvli, "#034b") == "0b" + instruction
                except AssertionError:
                    print("vsetvli error")
                    print(split_instruction)
                    print(f"Expected: {format(vsetvli, '#034b')}")
                    print(f"Got:      0b{instruction}")
                    exit(1)
            elif split_instruction[0] == "vle64.v":
                # "vle64.v": {
                #     "nf": bin(0),
                #     "mew": bin(0),
                #     "mop": bin(0),
                #     "vm": bin(0),
                #     "lumop": bin(0),
                #     "rs1": bin(0),
                #     "width": bin(0),
                #     "vd": bin(0),
                #     "opcode": bin(111),
                # }
                instruction = (
                    "000"
                    + "000"
                    + "1"
                    + "00000"
                    + format(
                        register_map(
                            split_instruction[2]
                            .replace("(", "")
                            .replace(")", "")
                        ),
                        "05b",
                    )
                    + "111"
                    + format(register_map(split_instruction[1]), "05b")
                    + "0000111"
                )
                try:
                    assert (
                        format(vle[vle_counter], "#034b") == "0b" + instruction
                    )
                except AssertionError:
                    print("vle64.v error")
                    print(split_instruction)
                    print(f"Expected: {format(vle[vle_counter], '#034b')}")
                    print(f"Got:      0b{instruction}")
                    exit(1)
                vle_counter += 1
            elif split_instruction[0] == "vsha2ms.vv":
                # "vsha2ms.vv": {
                #     "opcode": bin(101101),
                #     "": bin(1),
                #     "vs2": bin(0),
                #     "vs1": bin(0),
                #     "OPMVV": bin(10),
                #     "vd": bin(0),
                #     "OP-P": bin(1110111),
                # }
                instruction = (
                    "101101"
                    + "1"
                    + format(register_map(split_instruction[2]), "05b")
                    + format(register_map(split_instruction[3]), "05b")
                    + "010"
                    + format(register_map(split_instruction[1]), "05b")
                    + "1110111"
                )

            instruction = ".word 0x" + str(hex(int(instruction, 2))).upper()[2:]
            processed_instructions.append(instruction)
    with open("vector_hash_processed.s", "w") as f:
        for instruction in processed_instructions:
            f.write(f"{instruction}\n")
