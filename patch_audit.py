import sys

with open("tools/validation/runtime_bridge_audit.sh", "r") as f:
    content = f.read()

# Replace direct greps with PAYLOAD_OUTPUT extraction
new_content = content.replace('log_info "Auditing Runtime_Bridge syscall path: $TRACE_LOG"', '''log_info "Auditing Runtime_Bridge syscall path: $TRACE_LOG"

# Extract payload output interleaved with syscall markers
PAYLOAD_OUTPUT=$(grep -A1 "P10_SYSCALL_ENTER" "$TRACE_LOG" 2>/dev/null | \\
                 grep -v "P10_SYSCALL_ENTER" | \\
                 grep -v "^--$" | \\
                 grep -v "^\\[\\[" | \\
                 sed 's/\\[\\[AYKEN_SYSCALL_RETURN\\]\\]//g' | \\
                 tr -d '\\n' || echo "")''')

new_content = new_content.replace('grep -c "\\[U\\]\\[RUNTIME', 'echo "$PAYLOAD_OUTPUT" | grep -c "\\[U\\]\\[RUNTIME')
new_content = new_content.replace('"$TRACE_LOG" 2>/dev/null', '2>/dev/null')

new_content = new_content.replace('SYSCALL_EXIT=$(grep -c "\\[\\[AYKEN_SYSCALL_EXIT\\]\\]" "$TRACE_LOG" 2>/dev/null || echo "0")', 'SYSCALL_EXIT=$(grep -c "\\[\\[AYKEN_SYSCALL_RETURN\\]\\]" "$TRACE_LOG" 2>/dev/null || echo "0")')

with open("tools/validation/runtime_bridge_audit.sh", "w") as f:
    f.write(new_content)
