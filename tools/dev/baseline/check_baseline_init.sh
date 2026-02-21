#!/bin/bash
# Baseline init workflow durumunu kontrol et

REPO="kenanay/AykenOS"
WORKFLOW="perf-baseline-init.yml"

echo "🔍 Baseline Init Workflow Durumu"
echo "================================="
echo ""

# Son workflow run'ları al
curl -s "https://api.github.com/repos/${REPO}/actions/workflows/${WORKFLOW}/runs?per_page=5" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    runs = data.get('workflow_runs', [])
    
    if not runs:
        print('❌ Henüz workflow run bulunamadı')
        print('')
        print('Workflow başlatmak için:')
        print('https://github.com/${REPO}/actions/workflows/${WORKFLOW}')
        sys.exit(0)
    
    print(f'Son {len(runs)} workflow run:')
    print('')
    
    for i, run in enumerate(runs, 1):
        status = run['status']
        conclusion = run.get('conclusion', 'running')
        created = run['created_at']
        run_id = run['id']
        
        if conclusion == 'success':
            icon = '✅'
        elif conclusion == 'failure':
            icon = '❌'
        elif conclusion is None:
            icon = '⏳'
        else:
            icon = '⏭️'
        
        print(f'{i}. {icon} Run #{run_id}')
        print(f'   Status: {status}')
        print(f'   Conclusion: {conclusion}')
        print(f'   Created: {created}')
        print(f'   URL: {run[\"html_url\"]}')
        print('')
        
        # İlk run'ın detaylarını göster
        if i == 1 and status == 'completed':
            print('📦 Artifacts:')
            artifacts_url = f'https://api.github.com/repos/${REPO}/actions/runs/{run_id}/artifacts'
            import urllib.request
            with urllib.request.urlopen(artifacts_url) as response:
                artifacts = json.load(response)
                for artifact in artifacts.get('artifacts', []):
                    print(f'   - {artifact[\"name\"]} ({artifact[\"size_in_bytes\"]} bytes)')
                    print(f'     Download: {artifact[\"archive_download_url\"]}')
            print('')
except Exception as e:
    print(f'Error: {e}')
" 2>/dev/null

echo ""
echo "🌐 Workflow URL:"
echo "https://github.com/${REPO}/actions/workflows/${WORKFLOW}"
