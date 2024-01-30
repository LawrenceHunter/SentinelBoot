import sys

undefined_opcodes = ["vsetvli", "vle64.v", "vsha2ms.vv", "mv"]


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
    with open(sys.argv[1], "r") as f:
        instructions = []
        for line in f.readlines():
            if line.strip()[0] == "#":
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
            elif split_instruction[0] == "mv":
                # Not really unknown but the assembler cannot handle vx registers
                # "mv": {
                #     "opcode": bin(101101),
                #     "funct3": bin(0),
                #     "funct7": bin(0),
                #     "rs1": bin(0),
                #     "rs2": bin(10),
                #     "vd": bin(0)
                # }
                instruction = (
                    "0000000"
                    + format(register_map(split_instruction[2]), "05b")
                    + format(register_map(split_instruction[1]), "05b")
                    + "000"
                    + format(register_map("zero"), "05b")
                    + "0110011"
                )
            instruction = ".word 0x" + str(hex(int(instruction, 2))).upper()[2:]
            processed_instructions.append(instruction)
    split = sys.argv[1].split(".")
    with open("." + split[1] + ".rs", "w") as f:
        f.write("// AUTOGENERATED DO NOT EDIT\n\n")
        f.write("use core::arch::asm;\n\n")
        f.write(
            "pub unsafe fn hash_kernel_vcrypto(kernel_size: u64, kernel_pointer: u64, result: &mut [u64]) {\n"
        )
        f.write("\tasm!(\n")
        for instruction in processed_instructions:
            f.write(f'\t\t"{instruction}",\n')
        f.write("\t\tKERNEL_POINTER = in(reg) kernel_pointer,\n")
        f.write("\t\tkernel_size = in(reg) kernel_size,\n")
        f.write("\t\tout_1 = out(reg) result[0],\n")
        f.write("\t\tout_2 = out(reg) result[1],\n")
        f.write("\t\tout_3 = out(reg) result[2],\n")
        f.write("\t\tout_4 = out(reg) result[3]\n")
        f.write("\t);\n")
        f.write("}")
