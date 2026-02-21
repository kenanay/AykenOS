#!/bin/bash
# CI durumunu kontrol et

RUN_ID="22248809327"
API_URL="https://api.github.com/repos/kenanay/AykenOS/actions/runs/${RUN_ID}"

echo "🔍 CI Durumu Kontrol Ediliyor..."
echo "================================"
echo ""

# Genel durum
STATUS=$(curl -s "${API_URL}" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    print(f'Status: {data[\"status\"]}')
    print(f'Conclusion: {data.get(\"conclusion\", \"pending\")}')
    print(f'Started: {data[\"created_at\"]}')
    print(f'Updated: {data[\"updated_at\"]}')
except:
    print('Error fetching status')
" 2>/dev/null)

echo "$STATUS"
echo ""

# Job detayları
echo "📋 Job Detayları:"
echo "----------------"
curl -s "${API_URL}/jobs" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    for job in data.get('jobs', []):
        status = job['status']
        conclusion = job.get('conclusion', 'running')
        name = job['name']
        
        if conclusion == 'success':
            icon = '✅'
        elif conclusion == 'failure':
            icon = '❌'
        elif conclusion == 'skipped':
            icon = '⏭️'
        else:
            icon = '⏳'
        
        print(f'{icon} {name}: {status} - {conclusion}')
        
        if conclusion == 'failure':
            print(f'   URL: {job[\"html_url\"]}')
except:
    print('Error fetching jobs')
" 2>/dev/null

echo ""
echo "🌐 Web URL: https://github.com/kenanay/AykenOS/actions/runs/${RUN_ID}"
