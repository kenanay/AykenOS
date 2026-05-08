/**
 * AykenOS Dev Loop Observability Dashboard
 * 
 * Purpose: Read-only visualization of validation evidence
 * Authority: ZERO - purely observational
 * 
 * Maintainer: Kenan AY — System Architect
 */

// Configuration
const EVIDENCE_BASE = '../../out/evidence';
const LOGS_BASE = '../../out/logs';

// State
let currentRun = null;
let allRuns = [];

/**
 * Initialize dashboard
 */
async function init() {
    console.log('[Dashboard] Initializing...');
    
    // Load available runs
    await loadRuns();
    
    // Set up event listeners
    document.getElementById('runSelect').addEventListener('change', handleRunChange);
    document.getElementById('refreshBtn').addEventListener('click', handleRefresh);
    
    // Load most recent run if available
    if (allRuns.length > 0) {
        currentRun = allRuns[0];
        document.getElementById('runSelect').value = currentRun;
        await loadRunData(currentRun);
    }
    
    console.log('[Dashboard] Initialized');
}

/**
 * Load available runs from evidence directory
 */
async function loadRuns() {
    console.log('[Dashboard] Loading runs...');
    
    try {
        // In a real implementation, this would scan the evidence directory
        // For now, we'll use a mock implementation that checks for evidence
        const runs = await scanEvidenceDirectory();
        
        allRuns = runs.sort().reverse(); // Most recent first
        
        const select = document.getElementById('runSelect');
        select.innerHTML = '';
        
        if (allRuns.length === 0) {
            select.innerHTML = '<option value="">No runs available</option>';
            showEmptyState();
        } else {
            allRuns.forEach(run => {
                const option = document.createElement('option');
                option.value = run;
                option.textContent = formatRunId(run);
                select.appendChild(option);
            });
        }
        
        console.log(`[Dashboard] Loaded ${allRuns.length} runs`);
    } catch (error) {
        console.error('[Dashboard] Error loading runs:', error);
        showError('Failed to load runs');
    }
}

/**
 * Scan evidence directory for available runs
 */
async function scanEvidenceDirectory() {
    // This is a mock implementation
    // In production, this would use a backend API or file listing
    
    // For demonstration, return empty array
    // The actual implementation would scan out/evidence/run-* directories
    return [];
}

/**
 * Load data for a specific run
 */
async function loadRunData(runId) {
    console.log(`[Dashboard] Loading run: ${runId}`);
    
    try {
        // Load all evidence artifacts
        const [meta, summary, markers, perf, logs] = await Promise.all([
            loadMetadata(runId),
            loadSummary(runId),
            loadMarkers(runId),
            loadPerformance(runId),
            loadLogs(runId)
        ]);
        
        // Update UI
        updateStatusCard(summary);
        updateMarkerCard(markers);
        updatePerformanceCard(perf);
        updateContextCard(meta);
        updateLogViewer(logs);
        updateHistory();
        
        console.log('[Dashboard] Run data loaded');
    } catch (error) {
        console.error('[Dashboard] Error loading run data:', error);
        showError(`Failed to load run data: ${error.message}`);
    }
}

/**
 * Load metadata for a run
 */
async function loadMetadata(runId) {
    const path = `${EVIDENCE_BASE}/${runId}/meta/run.json`;
    
    try {
        const response = await fetch(path);
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
        }
        return await response.json();
    } catch (error) {
        console.warn(`[Dashboard] Could not load metadata: ${error.message}`);
        return {
            run_id: runId,
            timestamp: 'unknown',
            source: 'unknown',
            deterministic: false
        };
    }
}

/**
 * Load summary for a run
 */
async function loadSummary(runId) {
    const path = `${EVIDENCE_BASE}/${runId}/reports/summary.json`;
    
    try {
        const response = await fetch(path);
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
        }
        return await response.json();
    } catch (error) {
        console.warn(`[Dashboard] Could not load summary: ${error.message}`);
        return {
            boot: 'UNKNOWN',
            markers_ok: false,
            fail_closed: false,
            perf_regression: false
        };
    }
}

/**
 * Load markers for a run
 */
async function loadMarkers(runId) {
    const path = `${EVIDENCE_BASE}/${runId}/reports/markers.json`;
    
    try {
        const response = await fetch(path);
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
        }
        return await response.json();
    } catch (error) {
        console.warn(`[Dashboard] Could not load markers: ${error.message}`);
        return {
            EARLY_BOOT_OK: false,
            LATE_INIT_END: false,
            BOOT_OK: false,
            FAIL_CLOSED: false
        };
    }
}

/**
 * Load performance data for a run
 */
async function loadPerformance(runId) {
    const path = `${EVIDENCE_BASE}/${runId}/reports/perf.json`;
    
    try {
        const response = await fetch(path);
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
        }
        return await response.json();
    } catch (error) {
        console.warn(`[Dashboard] Could not load performance: ${error.message}`);
        return {
            boot_time_proxy: 0,
            method: 'unknown',
            valid: false,
            disclaimer: 'Performance data not available',
            unit: 'unknown'
        };
    }
}

/**
 * Load logs for a run
 */
async function loadLogs(runId) {
    const path = `${EVIDENCE_BASE}/${runId}/logs/boot.log`;
    
    try {
        const response = await fetch(path);
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
        }
        return await response.text();
    } catch (error) {
        console.warn(`[Dashboard] Could not load logs: ${error.message}`);
        
        // Try fallback to main logs directory
        try {
            const fallbackPath = `${LOGS_BASE}/boot_watch.log`;
            const fallbackResponse = await fetch(fallbackPath);
            if (fallbackResponse.ok) {
                return await fallbackResponse.text();
            }
        } catch (fallbackError) {
            console.warn(`[Dashboard] Fallback log load failed: ${fallbackError.message}`);
        }
        
        return 'Log data not available';
    }
}

/**
 * Update status card
 */
function updateStatusCard(summary) {
    const statusBadge = document.getElementById('bootStatus');
    const validationResult = document.getElementById('validationResult');
    const runId = document.getElementById('runId');
    
    // Update status badge
    statusBadge.textContent = summary.boot || 'UNKNOWN';
    statusBadge.className = 'status-badge';
    
    if (summary.boot === 'PASS') {
        statusBadge.classList.add('status-pass');
    } else if (summary.boot === 'FAIL') {
        statusBadge.classList.add('status-fail');
    } else {
        statusBadge.classList.add('status-unknown');
    }
    
    // Update validation result
    validationResult.textContent = summary.boot || 'UNKNOWN';
    
    // Update run ID
    runId.textContent = currentRun || '—';
}

/**
 * Update marker card
 */
function updateMarkerCard(markers) {
    const markerStatus = document.getElementById('markerStatus');
    const markerList = document.getElementById('markerList');
    
    // Determine overall marker status
    const allPresent = markers.EARLY_BOOT_OK && markers.LATE_INIT_END && markers.BOOT_OK;
    
    markerStatus.textContent = allPresent ? 'ALL PRESENT' : 'INCOMPLETE';
    markerStatus.className = 'status-badge';
    markerStatus.classList.add(allPresent ? 'status-pass' : 'status-fail');
    
    // Build marker list
    const markerData = [
        { name: 'EARLY_BOOT_OK', present: markers.EARLY_BOOT_OK, label: '[K][EARLY_BOOT_OK]' },
        { name: 'LATE_INIT_END', present: markers.LATE_INIT_END, label: '[K][LATE_INIT_END]' },
        { name: 'BOOT_OK', present: markers.BOOT_OK, label: '[[AYKEN_BOOT_OK]]' },
        { name: 'FAIL_CLOSED', present: markers.FAIL_CLOSED, label: '[VCP][FAIL_CLOSED]' }
    ];
    
    markerList.innerHTML = '';
    markerData.forEach(marker => {
        const li = document.createElement('li');
        li.className = 'marker-item';
        
        const icon = document.createElement('span');
        icon.className = 'marker-icon';
        icon.textContent = marker.present ? '✅' : '❌';
        
        const name = document.createElement('span');
        name.className = 'marker-name';
        name.textContent = marker.label;
        
        li.appendChild(icon);
        li.appendChild(name);
        markerList.appendChild(li);
    });
}

/**
 * Update performance card
 */
function updatePerformanceCard(perf) {
    const perfStatus = document.getElementById('perfStatus');
    const perfValue = document.getElementById('perfValue');
    const perfBar = document.getElementById('perfBar');
    const perfLabel = document.getElementById('perfLabel');
    
    // Update status
    perfStatus.textContent = perf.valid ? 'VALID' : 'INVALID';
    perfStatus.className = 'status-badge';
    perfStatus.classList.add(perf.valid ? 'status-pass' : 'status-unknown');
    
    // Update value
    if (perf.valid && perf.boot_time_proxy > 0) {
        perfValue.textContent = `${perf.boot_time_proxy} ${perf.unit || 'lines'}`;
        
        // Update bar (normalize to 0-100%)
        // Assume 2000 lines is 100%
        const percentage = Math.min((perf.boot_time_proxy / 2000) * 100, 100);
        perfBar.style.width = `${percentage}%`;
        perfLabel.textContent = `${Math.round(percentage)}%`;
        
        // Color based on performance
        perfBar.className = 'perf-bar-fill';
        if (percentage > 80) {
            perfBar.classList.add('danger');
        } else if (percentage > 60) {
            perfBar.classList.add('warning');
        }
    } else {
        perfValue.textContent = '—';
        perfBar.style.width = '0%';
        perfLabel.textContent = 'N/A';
    }
}

/**
 * Update context card
 */
function updateContextCard(meta) {
    document.getElementById('contextTimestamp').textContent = 
        meta.time_utc || meta.timestamp || '—';
    document.getElementById('contextSource').textContent = 
        meta.source || 'unknown';
    document.getElementById('contextGitSha').textContent = 
        meta.git_sha || '—';
    document.getElementById('contextDeterministic').textContent = 
        meta.deterministic ? '✅ Yes' : '❌ No';
}

/**
 * Update log viewer
 */
function updateLogViewer(logs) {
    const logViewer = document.getElementById('logViewer');
    
    if (!logs || logs === 'Log data not available') {
        logViewer.innerHTML = `
            <div class="empty-state">
                <div class="empty-state-icon">📋</div>
                <div>Log data not available</div>
            </div>
        `;
        return;
    }
    
    // Parse and format logs
    const lines = logs.split('\n');
    const formattedLines = lines.map(line => {
        let className = 'log-line';
        
        // Visual differentiation based on content
        if (line.includes('[K][EARLY_BOOT_OK]') || 
            line.includes('[K][LATE_INIT_END]') || 
            line.includes('[[AYKEN_BOOT_OK]]')) {
            className = 'log-line marker';
        } else if (line.toLowerCase().includes('error') || 
                   line.toLowerCase().includes('panic')) {
            className = 'log-line error';
        } else if (line.toLowerCase().includes('warning') || 
                   line.toLowerCase().includes('warn')) {
            className = 'log-line warning';
        }
        
        return `<div class="${className}">${escapeHtml(line)}</div>`;
    });
    
    logViewer.innerHTML = formattedLines.join('');
}

/**
 * Update history table
 */
function updateHistory() {
    const historyTable = document.getElementById('historyTable');
    
    if (allRuns.length === 0) {
        historyTable.innerHTML = `
            <tr>
                <td colspan="5" style="text-align: center; color: #6e7681;">No runs available</td>
            </tr>
        `;
        return;
    }
    
    // For now, show placeholder
    // In production, this would load summary data for all runs
    historyTable.innerHTML = `
        <tr>
            <td colspan="5" style="text-align: center; color: #6e7681;">
                History aggregation not yet implemented
            </td>
        </tr>
    `;
}

/**
 * Handle run selection change
 */
async function handleRunChange(event) {
    const runId = event.target.value;
    if (runId) {
        currentRun = runId;
        await loadRunData(runId);
    }
}

/**
 * Handle refresh button click
 */
async function handleRefresh() {
    console.log('[Dashboard] Refreshing...');
    await loadRuns();
    if (currentRun) {
        await loadRunData(currentRun);
    }
}

/**
 * Show empty state
 */
function showEmptyState() {
    document.getElementById('bootStatus').textContent = 'NO DATA';
    document.getElementById('bootStatus').className = 'status-badge status-unknown';
    document.getElementById('validationResult').textContent = '—';
    document.getElementById('runId').textContent = '—';
    
    document.getElementById('markerStatus').textContent = 'NO DATA';
    document.getElementById('markerStatus').className = 'status-badge status-unknown';
    document.getElementById('markerList').innerHTML = `
        <li class="marker-item">
            <span class="marker-icon">⏳</span>
            <span class="marker-name">No data available</span>
        </li>
    `;
    
    document.getElementById('perfStatus').textContent = 'NO DATA';
    document.getElementById('perfStatus').className = 'status-badge status-unknown';
    document.getElementById('perfValue').textContent = '—';
    
    document.getElementById('logViewer').innerHTML = `
        <div class="empty-state">
            <div class="empty-state-icon">📋</div>
            <div>No runs available. Run dev_loop.sh to generate evidence.</div>
        </div>
    `;
}

/**
 * Show error message
 */
function showError(message) {
    console.error(`[Dashboard] Error: ${message}`);
    // Could show a toast notification here
}

/**
 * Format run ID for display
 */
function formatRunId(runId) {
    // Extract timestamp from run ID
    const match = runId.match(/run-(\d{8}T\d{6}Z)/);
    if (match) {
        const timestamp = match[1];
        return `${timestamp.substring(0, 8)} ${timestamp.substring(9, 15)}`;
    }
    return runId;
}

/**
 * Escape HTML to prevent XSS
 */
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Initialize on load
document.addEventListener('DOMContentLoaded', init);
