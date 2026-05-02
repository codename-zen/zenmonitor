/**
 * ZenMonitor Dashboard - Main Application JavaScript
 * Uses Server-Sent Events for real-time updates
 */

// ─── State ──────────────────────────────────────────────────────────────────

const state = {
    monitors: [],
    agents: [],
    latestResults: {},  // monitor_id -> last result
    charts: {},
    sse: null,
};

// ─── Initialization ─────────────────────────────────────────────────────────

document.addEventListener('DOMContentLoaded', () => {
    loadMonitors();
    loadAgents();
    initSSE();
    initCharts();
});

// ─── API Calls ──────────────────────────────────────────────────────────────

async function loadMonitors() {
    try {
        const resp = await fetch('/api/monitors');
        const data = await resp.json();
        if (data.success && data.data) {
            state.monitors = data.data;
            renderMonitors();
            updateStats();
        }
    } catch (e) {
        console.error('Failed to load monitors:', e);
    }
}

async function loadAgents() {
    try {
        const resp = await fetch('/api/agents');
        const data = await resp.json();
        if (data.success && data.data) {
            state.agents = data.data;
            renderAgents();
            updateStats();
        }
    } catch (e) {
        console.error('Failed to load agents:', e);
    }
}

async function submitAddMonitor(event) {
    event.preventDefault();
    const form = event.target;
    const formData = new FormData(form);

    const payload = {
        name: formData.get('name'),
        monitor_type: formData.get('monitor_type'),
        target: formData.get('target'),
        port: formData.get('port') ? parseInt(formData.get('port')) : null,
        interval_seconds: parseInt(formData.get('interval_seconds')) || 60,
    };

    try {
        const resp = await fetch('/api/monitors', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload),
        });
        const data = await resp.json();
        if (data.success) {
            state.monitors.push(data.data);
            renderMonitors();
            updateStats();
            closeAddMonitorModal();
            form.reset();
        } else {
            alert('Error: ' + (data.error || 'Unknown error'));
        }
    } catch (e) {
        alert('Failed to create monitor: ' + e.message);
    }
}

async function deleteMonitor(id) {
    if (!confirm('Delete this monitor?')) return;

    try {
        const resp = await fetch(`/api/monitors/${id}`, { method: 'DELETE' });
        const data = await resp.json();
        if (data.success) {
            state.monitors = state.monitors.filter(m => m.id !== id);
            renderMonitors();
            updateStats();
        }
    } catch (e) {
        alert('Failed to delete monitor: ' + e.message);
    }
}

// ─── Server-Sent Events ─────────────────────────────────────────────────────

function initSSE() {
    state.sse = new EventSource('/api/events');

    state.sse.onopen = () => {
        document.getElementById('connection-status').innerHTML = `
            <span class="w-2 h-2 rounded-full bg-zen-success mr-2 pulse-dot"></span>
            Connected
        `;
    };

    state.sse.onerror = () => {
        document.getElementById('connection-status').innerHTML = `
            <span class="w-2 h-2 rounded-full bg-zen-danger mr-2"></span>
            Disconnected
        `;
        // Reconnect after 5 seconds
        setTimeout(() => {
            if (state.sse.readyState === EventSource.CLOSED) {
                initSSE();
            }
        }, 5000);
    };

    state.sse.onmessage = (event) => {
        try {
            const data = JSON.parse(event.data);
            handleSSEEvent(data);
        } catch (e) {
            console.error('Failed to parse SSE event:', e);
        }
    };
}

function handleSSEEvent(event) {
    switch (event.type) {
        case 'CheckResult':
            state.latestResults[event.data.monitor_id] = event.data;
            updateMonitorCard(event.data.monitor_id, event.data);
            updateStats();
            break;
        case 'AgentMetrics':
            updateAgentMetrics(event.data);
            break;
        case 'MonitorUpdate':
            loadMonitors(); // Reload full list
            break;
        case 'AgentStatus':
            loadAgents();
            break;
    }
}

// ─── Rendering ──────────────────────────────────────────────────────────────

function renderMonitors() {
    const container = document.getElementById('monitors-list');

    if (state.monitors.length === 0) {
        container.innerHTML = `
            <div class="bg-zen-card border border-zen-border border-dashed rounded-xl p-8 flex items-center justify-center col-span-full">
                <p class="text-zen-muted text-sm">No monitors configured. Click "Add Monitor" to get started.</p>
            </div>
        `;
        return;
    }

    container.innerHTML = state.monitors.map(monitor => {
        const result = state.latestResults[monitor.id];
        const statusClass = result ? `status-${result.status}` : 'status-unknown';
        const statusText = result ? result.status.toUpperCase() : 'PENDING';
        const responseTime = result?.response_time_ms ? `${result.response_time_ms.toFixed(1)}ms` : '-';

        return `
            <div class="bg-zen-card border border-zen-border rounded-xl p-4 card-hover transition-all" id="monitor-${monitor.id}">
                <div class="flex items-start justify-between mb-3">
                    <div>
                        <h3 class="text-white font-medium">${escapeHtml(monitor.name)}</h3>
                        <p class="text-zen-muted text-xs mt-0.5">${escapeHtml(monitor.target)}</p>
                    </div>
                    <button onclick="deleteMonitor('${monitor.id}')" class="text-zen-muted hover:text-zen-danger text-sm">✕</button>
                </div>
                <div class="flex items-center justify-between">
                    <div class="flex items-center space-x-2">
                        <span class="w-2 h-2 rounded-full ${statusClass === 'status-up' ? 'bg-zen-success' : statusClass === 'status-down' ? 'bg-zen-danger' : 'bg-zen-warning'}"></span>
                        <span class="${statusClass} text-sm font-medium">${statusText}</span>
                    </div>
                    <div class="text-right">
                        <span class="text-zen-muted text-xs">${responseTime}</span>
                        <span class="text-zen-muted text-xs ml-2 px-1.5 py-0.5 bg-zen-bg rounded">${monitor.monitor_type}</span>
                    </div>
                </div>
            </div>
        `;
    }).join('');
}

function renderAgents() {
    const container = document.getElementById('agents-list');

    if (state.agents.length === 0) {
        container.innerHTML = `
            <div class="bg-zen-card border border-zen-border border-dashed rounded-xl p-8 flex items-center justify-center col-span-full">
                <p class="text-zen-muted text-sm">No agents connected. Deploy the zenmonitor-agent to your servers.</p>
            </div>
        `;
        return;
    }

    container.innerHTML = state.agents.map(agent => `
        <div class="bg-zen-card border border-zen-border rounded-xl p-4 card-hover transition-all">
            <div class="flex items-center space-x-3 mb-3">
                <div class="w-10 h-10 bg-zen-accent/20 rounded-lg flex items-center justify-center">
                    <span class="text-zen-accent text-lg">🖥</span>
                </div>
                <div>
                    <h3 class="text-white font-medium">${escapeHtml(agent.hostname)}</h3>
                    <p class="text-zen-muted text-xs">${escapeHtml(agent.ip_address || 'Unknown IP')}</p>
                </div>
            </div>
            <div class="grid grid-cols-2 gap-2 text-xs">
                <div class="bg-zen-bg rounded px-2 py-1">
                    <span class="text-zen-muted">OS:</span>
                    <span class="text-white ml-1">${escapeHtml(agent.os || 'N/A')}</span>
                </div>
                <div class="bg-zen-bg rounded px-2 py-1">
                    <span class="text-zen-muted">Kernel:</span>
                    <span class="text-white ml-1">${escapeHtml(agent.kernel || 'N/A')}</span>
                </div>
            </div>
        </div>
    `).join('');
}

function updateMonitorCard(monitorId, result) {
    const card = document.getElementById(`monitor-${monitorId}`);
    if (card) {
        // Re-render just this card's status
        renderMonitors();
    }
}

function updateAgentMetrics(data) {
    // Update charts with new agent data
    if (state.charts.cpu) {
        const chart = state.charts.cpu;
        const now = new Date().toLocaleTimeString();
        chart.data.labels.push(now);
        chart.data.datasets[0].data.push(data.cpu_usage);
        if (chart.data.labels.length > 30) {
            chart.data.labels.shift();
            chart.data.datasets[0].data.shift();
        }
        chart.update('none');
    }

    if (state.charts.memory) {
        const chart = state.charts.memory;
        const now = new Date().toLocaleTimeString();
        chart.data.labels.push(now);
        chart.data.datasets[0].data.push(data.ram_used_mb);
        chart.data.datasets[1].data.push(data.ram_available_mb);
        if (chart.data.labels.length > 30) {
            chart.data.labels.shift();
            chart.data.datasets[0].data.shift();
            chart.data.datasets[1].data.shift();
        }
        chart.update('none');
    }
}

function updateStats() {
    document.getElementById('stat-total').textContent = state.monitors.length;

    let up = 0, down = 0;
    for (const monitor of state.monitors) {
        const result = state.latestResults[monitor.id];
        if (result) {
            if (result.status === 'up') up++;
            else if (result.status === 'down') down++;
        }
    }
    document.getElementById('stat-up').textContent = up;
    document.getElementById('stat-down').textContent = down;
    document.getElementById('stat-agents').textContent = state.agents.length;
}

// ─── Charts ─────────────────────────────────────────────────────────────────

function initCharts() {
    const chartDefaults = {
        responsive: true,
        plugins: {
            legend: { labels: { color: '#94a3b8' } },
        },
        scales: {
            x: { ticks: { color: '#94a3b8' }, grid: { color: '#2a2d3e' } },
            y: { ticks: { color: '#94a3b8' }, grid: { color: '#2a2d3e' } },
        },
    };

    // Response Times Chart
    const rtCtx = document.getElementById('chart-response-times');
    if (rtCtx) {
        state.charts.responseTime = new Chart(rtCtx, {
            type: 'line',
            data: {
                labels: [],
                datasets: [{
                    label: 'Response Time (ms)',
                    data: [],
                    borderColor: '#6366f1',
                    backgroundColor: 'rgba(99, 102, 241, 0.1)',
                    fill: true,
                    tension: 0.3,
                }],
            },
            options: chartDefaults,
        });
    }

    // CPU Usage Chart
    const cpuCtx = document.getElementById('chart-cpu-usage');
    if (cpuCtx) {
        state.charts.cpu = new Chart(cpuCtx, {
            type: 'line',
            data: {
                labels: [],
                datasets: [{
                    label: 'CPU %',
                    data: [],
                    borderColor: '#10b981',
                    backgroundColor: 'rgba(16, 185, 129, 0.1)',
                    fill: true,
                    tension: 0.3,
                }],
            },
            options: { ...chartDefaults, scales: { ...chartDefaults.scales, y: { ...chartDefaults.scales.y, min: 0, max: 100 } } },
        });
    }

    // Memory Usage Chart
    const memCtx = document.getElementById('chart-memory-usage');
    if (memCtx) {
        state.charts.memory = new Chart(memCtx, {
            type: 'line',
            data: {
                labels: [],
                datasets: [
                    {
                        label: 'Used (MB)',
                        data: [],
                        borderColor: '#ef4444',
                        backgroundColor: 'rgba(239, 68, 68, 0.1)',
                        fill: true,
                        tension: 0.3,
                    },
                    {
                        label: 'Available (MB)',
                        data: [],
                        borderColor: '#10b981',
                        backgroundColor: 'rgba(16, 185, 129, 0.1)',
                        fill: true,
                        tension: 0.3,
                    },
                ],
            },
            options: chartDefaults,
        });
    }

    // Uptime Chart (placeholder)
    const uptimeCtx = document.getElementById('chart-uptime');
    if (uptimeCtx) {
        state.charts.uptime = new Chart(uptimeCtx, {
            type: 'bar',
            data: {
                labels: [],
                datasets: [{
                    label: 'Uptime %',
                    data: [],
                    backgroundColor: '#6366f1',
                    borderRadius: 4,
                }],
            },
            options: { ...chartDefaults, scales: { ...chartDefaults.scales, y: { ...chartDefaults.scales.y, min: 0, max: 100 } } },
        });
    }
}

// ─── UI Helpers ─────────────────────────────────────────────────────────────

function switchTab(tab) {
    const tabs = ['monitors', 'agents', 'charts'];
    tabs.forEach(t => {
        document.getElementById(`panel-${t}`).classList.toggle('hidden', t !== tab);
        const tabBtn = document.getElementById(`tab-${t}`);
        if (t === tab) {
            tabBtn.classList.add('text-zen-accent', 'border-zen-accent');
            tabBtn.classList.remove('text-zen-muted', 'border-transparent');
        } else {
            tabBtn.classList.remove('text-zen-accent', 'border-zen-accent');
            tabBtn.classList.add('text-zen-muted', 'border-transparent');
        }
    });
}

function openAddMonitorModal() {
    document.getElementById('modal-add-monitor').classList.remove('hidden');
}

function closeAddMonitorModal() {
    document.getElementById('modal-add-monitor').classList.add('hidden');
}

function escapeHtml(text) {
    if (!text) return '';
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Close modal on backdrop click
document.getElementById('modal-add-monitor')?.addEventListener('click', (e) => {
    if (e.target === e.currentTarget) closeAddMonitorModal();
});

// Close modal on Escape key
document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') closeAddMonitorModal();
});
