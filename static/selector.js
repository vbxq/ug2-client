let allBuilds = [];
let currentPage = 1;
const PAGE_SIZE = 50;

async function loadBuilds() {
    try {
        const res = await fetch('/api/builds');
        allBuilds = await res.json();
        currentPage = 1;
        renderBuilds();
        updateStatus();
    } catch (e) {
        showToast('Failed to load builds: ' + e.message, true);
    }
}

function renderBuilds() {
    const search = document.getElementById('search').value.toLowerCase();
    const filter = document.getElementById('filterPatched').value;

    let filtered = allBuilds.filter(b => {
        if (search && !b.build_hash.toLowerCase().includes(search)) return false;
        if (filter === 'patched' && !b.is_patched) return false;
        if (filter === 'pending' && b.is_patched) return false;
        return true;
    });

    const totalPages = Math.ceil(filtered.length / PAGE_SIZE);
    const start = (currentPage - 1) * PAGE_SIZE;
    const page = filtered.slice(start, start + PAGE_SIZE);

    const body = document.getElementById('buildsBody');
    if (page.length === 0) {
        body.innerHTML = '<tr><td colspan="5"><div class="empty-state">No builds found</div></td></tr>';
        document.getElementById('pagination').innerHTML = '';
        return;
    }

    body.innerHTML = page.map((b, i) => `
        <tr style="animation-delay:${i * 15}ms">
            <td><span class="hash" title="${b.build_hash}">${b.build_hash.slice(0, 12)}...</span></td>
            <td><span class="channel-tag">${b.channel}</span></td>
            <td><span class="date-cell">${new Date(b.build_date).toLocaleDateString()}</span></td>
            <td>
                <div class="badges">
                    ${b.is_active ? '<span class="badge badge-active">active</span>' : ''}
                    ${b.is_patched ? '<span class="badge badge-patched">patched</span>' : '<span class="badge badge-pending">pending</span>'}
                </div>
            </td>
            <td class="actions">
                ${!b.is_patched ? `<button class="button button-primary button-sm" onclick="downloadBuild('${b.build_hash}')">Download</button>` : ''}
                ${b.is_patched && !b.is_active ? `<button class="button button-green button-sm" onclick="activateBuild('${b.build_hash}')">Activate</button>` : ''}
                ${b.is_patched ? `<button class="button button-ghost button-sm" onclick="repatchBuild('${b.build_hash}')">Repatch</button>` : ''}
            </td>
        </tr>
    `).join('');

    let pagHtml = '';
    if (totalPages > 1) {
        for (let i = 1; i <= totalPages; i++) {
            pagHtml += `<button class="page-btn ${i === currentPage ? 'active' : ''}"
                onclick="currentPage=${i};renderBuilds()">${i}</button>`;
        }
    }
    document.getElementById('pagination').innerHTML = pagHtml;
}

function updateStatus() {
    const active = allBuilds.find(b => b.is_active);
    const dot = document.getElementById('statusDot');
    const text = document.getElementById('statusText');
    if (active) {
        dot.className = 'blob';
        text.innerHTML = 'Active build: <span class="status-hash">' + active.build_hash.slice(0, 16) + '...</span> - <a href="/" class="status-link">Open client</a>';
    } else {
        dot.className = 'blob inactive';
        text.textContent = 'No active build - download and activate one below';
    }
}

async function fetchCurrentBuild() {
    const btn = document.getElementById('fetchCurrentBtn');
    btn.disabled = true;
    btn.textContent = 'Fetching...';
    try {
        const res = await fetch('/api/builds/fetch-current', { method: 'POST' });
        const data = await res.json();
        if (data.status === 'error') {
            showToast(data.message, true);
            resetFetchBtn(btn);
            return;
        }
        showToast(data.message);
        const hashMatch = data.message.match(/([a-f0-9]{40})/);
        if (hashMatch) {
            pollBuildStatus(hashMatch[1]);
        }
        setTimeout(() => resetFetchBtn(btn), 5000);
    } catch (e) {
        showToast('Fetch failed: ' + e.message, true);
        resetFetchBtn(btn);
    }
}

function resetFetchBtn(btn) {
    btn.disabled = false;
    btn.textContent = 'Fetch Current Build';
}

async function downloadBuild(hash) {
    const btn = event.target.closest('button');
    btn.disabled = true;
    btn.textContent = 'Downloading...';
    try {
        const res = await fetch('/api/builds/download', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ build_hash: hash })
        });
        const data = await res.json();
        showToast(data.message);
        pollBuildStatus(hash);
    } catch (e) {
        showToast('Download failed: ' + e.message, true);
        btn.disabled = false;
        btn.textContent = 'Download';
    }
}

function pollBuildStatus(hash) {
    const interval = setInterval(async () => {
        const res = await fetch('/api/builds');
        const builds = await res.json();
        const build = builds.find(b => b.build_hash === hash);
        if (build && build.is_patched) {
            clearInterval(interval);
            allBuilds = builds;
            renderBuilds();
            updateStatus();
            showToast('Build ' + hash.slice(0, 12) + ' ready!');
        }
    }, 3000);
    setTimeout(() => clearInterval(interval), 300000);
}

async function activateBuild(hash) {
    try {
        const res = await fetch('/api/builds/active', {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ build_hash: hash })
        });
        const data = await res.json();
        if (data.status === 'ok') {
            showToast('Build activated! Opening client...');
            await loadBuilds();
            setTimeout(() => window.open('/', '_blank'), 500);
        } else {
            showToast(data.message, true);
        }
    } catch (e) {
        showToast('Activation failed: ' + e.message, true);
    }
}

async function repatchBuild(hash) {
    try {
        const res = await fetch(`/api/builds/${hash}/repatch`, { method: 'POST' });
        const data = await res.json();
        showToast(data.message);
    } catch (e) {
        showToast('Repatch failed: ' + e.message, true);
    }
}

function showToast(msg, isError = false) {
    const container = document.getElementById('toastContainer');
    const el = document.createElement('div');
    el.className = 'toast' + (isError ? ' error' : '');
    el.textContent = msg;
    container.appendChild(el);
    setTimeout(() => {
        el.classList.add('hide');
        setTimeout(() => el.remove(), 200);
    }, 4000);
}

let searchTimeout;
document.getElementById('search').addEventListener('input', () => {
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => { currentPage = 1; renderBuilds(); }, 200);
});
document.getElementById('filterPatched').addEventListener('change', () => { currentPage = 1; renderBuilds(); });

loadBuilds();
