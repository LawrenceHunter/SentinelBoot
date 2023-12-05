import os

pid = os.popen("pgrep qemu").read().split()
for item in pid:
    res = os.popen(f"find /proc/{item} -type f | grep tap0").read().split()
    res = res[0]
    res = int(res.split("/")[2])
    os.popen(f"kill -9 {res}")
