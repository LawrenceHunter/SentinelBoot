import serial

ser = serial.Serial("/dev/ttyUSB0", 115200, timeout=0.5)

tests = [
    (
        "❕ Waiting for 'Hit any key to stop autoboot'...",
        "✅ Got 'Hit any key to stop autoboot'",
        "".encode(),
        "Hit any key to stop autoboot",
    ),
    (
        "❕ Waiting for Kernel output...",
        "✅ Got kernel output",
        "".encode(),
        "[    0.000000]",
    ),
]

test = 0

if __name__ == "__main__":
    print(tests[test][0])
    while test < len(tests):
        while ser.in_waiting:
            data = ser.readline().decode("ascii")
            print(f"\t {data}")
            if tests[test][3] in data:
                state = 1
                print(tests[test][1])
                test += 1
                try:
                    print(tests[test][0])
                except IndexError:
                    break
                ser.write(tests[test][2])
    print("✅ All expected output achieved!")
