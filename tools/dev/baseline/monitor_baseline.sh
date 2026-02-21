#!/bin/bash
# Baseline init workflow'unu izle

RUN_ID="22248958636"
API_URL="https://api.github.com/repos/kenanay/AykenOS/actions/runs/${RUN_ID}"

echo "🔍 Baseline Init Workflow İzleniyor..."
echo "======================================="
echo "Run ID: ${RUN_ID}"
echo ""

while true; do
    STATUS=$(curl -s "${API_URL}" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    status = data['status']
    conclusion = data.get('conclusion', 'running')
    
    if status == 'completed':
        print(f'completed:{conclusion}')
    else:
        print(f'{status}:running')
except:
    print('error:unknown')
" 2>/dev/null)
    
    IFS=':' read -r status conclusion <<< "$STATUS"
    
    clear
    echo "🔍 Baseline Init Workflow İzleniyor..."
    echo "======================================="
    echo "Run ID: ${RUN_ID}"
    echo ""
    echo "Status: ${status}"
    echo "Conclusion: ${conclusion}"
    echo ""
    echo "URL: https://github.com/kenanay/AykenOS/actions/runs/${RUN_ID}"
    echo ""
    
    if [ "$status" = "completed" ]; then
        if [ "$conclusion" = "success" ]; then
            echo "✅ BAŞARILI! Artifacts indirmeye hazır."
            echo ""
            echo "Artifacts URL:"
            echo "https://github.com/kenanay/AykenOS/actions/runs/${RUN_ID}"
        else
            echo "❌ BAŞARISIZ: ${conclusion}"
        fi
        break
    fi
    
    echo "⏳ Bekliyor... (her 10 saniyede güncellenir)"
    sleep 10
done
