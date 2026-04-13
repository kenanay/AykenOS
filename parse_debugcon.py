import sys, re

with open('evidence/runtime-bridge-proof/qemu_kernel_trace_allowed.log') as f:
    text = f.read()

# We are looking for lines that are exactly 1 char long before [[AYKEN_SYSCALL_RETURN]]
res = []
lines = text.split('\n')
for i, line in enumerate(lines):
    if line == '[[AYKEN_SYSCALL_RETURN]]' and i > 0:
        prev = lines[i-1]
        if len(prev) == 1:
            res.append(prev)
print(''.join(res))
