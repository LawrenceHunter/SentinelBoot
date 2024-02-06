#  Set vector configuration
vsetivli x0, 4, e32, m1, ta, ma

# Load 512 bits of the message block into v10-v13 endian swaping
vle32.v v10, (a1)
vrev8.v v10, v10
vle32.v v11, (a1)
vrev8.v v11, v11
vle32.v v12, (a1)
vrev8.v v12, v12
vle32.v v13, (a1)
vrev8.v v13, v13

# Load H[0..8]
vle32.v v16, (a0)
vle32.v v17, (a0)

# Capture initial H to allow computing H'
vmv.v.v v26, v16
vmv.v.v v27, v17

# Set v0 for vmerge that replaces first word
vid.v v0
vmseq.vi v0, v0, 0x0

# Round 0
vle32.v v15, (a2)
vadd.vv v14, v15, v10
# vsha2cl.vv v17, v16, v14
# vsha2ch.vv v16, v17, v14
vmerge.vvm v14, v12, v11, v0
# vsha2ms.vv v10, v14, v13

# Round 1
vle32.v v15, (a2)
vadd.vv v14, v15, v11
vsha2cl.vv v17, v16, v14
vsha2ch.vv v16, v17, v14
vmerge.vvm v14, v13, v12, v0
vsha2ms.vv v11, v14, v10

# Round 2
vle32.v v15, (a2)
vadd.vv v14, v15, v12
# vsha2cl.vv v17, v16, v14
# vsha2ch.vv v16, v17, v14
vmerge.vvm v14, v10, v13, v0
# vsha2ms.vv v12, v14, v11

# Round 3
vle32.v v15, (a2)
vadd.vv v14, v15, v13
# vsha2cl.vv v17, v16, v14
# vsha2ch.vv v16, v17, v14
vmerge.vvm v14, v11, v10, v0
# vsha2ms.vv v13, v14, v12

# Round 4
vle32.v v15, (a2)
vadd.vv v14, v15, v10
# vsha2cl.vv v17, v16, v14
# vsha2ch.vv v16, v17, v14
vmerge.vvm v14, v12, v11, v0
# vsha2ms.vv v10, v14, v13

# Round 5
vle32.v v15, (a2)
vadd.vv v14, v15, v11
# vsha2cl.vv v17, v16, v14
# vsha2ch.vv v16, v17, v14
vmerge.vvm v14, v13, v12, v0
# vsha2ms.vv v11, v14, v10

# Round 6
vle32.v v15, (a2)
vadd.vv v14, v15, v12
# vsha2cl.vv v17, v16, v14
# vsha2ch.vv v16, v17, v14
vmerge.vvm v14, v10, v13, v0
# vsha2ms.vv v12, v14, v11

# Round 7
vle32.v v15, (a2)
vadd.vv v14, v15, v13
# vsha2cl.vv v17, v16, v14
# vsha2ch.vv v16, v17, v14
vmerge.vvm v14, v11, v10, v0
# vsha2ms.vv v13, v14, v12

# Round 8
vle32.v v15, (a2)
vadd.vv v14, v15, v10
# vsha2cl.vv v17, v16, v14
# vsha2ch.vv v16, v17, v14
vmerge.vvm v14, v12, v11, v0
# vsha2ms.vv v10, v14, v13

# Round 9
vle32.v v15, (a2)
vadd.vv v14, v15, v11
# vsha2cl.vv v17, v16, v14
# vsha2ch.vv v16, v17, v14
vmerge.vvm v14, v13, v12, v0
# vsha2ms.vv v11, v14, v10

# Round 10
vle32.v v15, (a2)
vadd.vv v14, v15, v12
# vsha2cl.vv v17, v16, v14
# vsha2ch.vv v16, v17, v14
vmerge.vvm v14, v10, v13, v0
# vsha2ms.vv v12, v14, v11

# Round 11
vle32.v v15, (a2)
vadd.vv v14, v15, v13
# vsha2cl.vv v17, v16, v14
# vsha2ch.vv v16, v17, v14
vmerge.vvm v14, v11, v10, v0
# vsha2ms.vv v13, v14, v12

# Round 12
vle32.v v15, (a2)
vadd.vv v14, v15, v10
# vsha2cl.vv v17, v16, v14
# vsha2ch.vv v16, v17, v14

# Round 13
vle32.v v15, (a2)
vadd.vv v14, v15, v11
# vsha2cl.vv v17, v16, v14
# vsha2ch.vv v16, v17, v14

# Round 14
vle32.v v15, (a2)
vadd.vv v14, v15, v12
# vsha2cl.vv v17, v16, v14
# vsha2ch.vv v16, v17, v14

# Round 15
vle32.v v15, (a2)
vadd.vv v14, v15, v13
# vsha2cl.vv v17, v16, v14
# vsha2ch.vv v16, v17, v14

# Update the hash
add.vv v16, v26, v16
vadd.vv v17, v27, v17

# Save the hash
vse32.v v17, (a0)
vse32.v v16, (a0)
