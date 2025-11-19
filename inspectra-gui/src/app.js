// Global State
let allProcesses = [];
let selectedPid = null;
let scanResults = [];
let filteredResults = [];
let addressList = [];
let previousScanResults = null;
let modalSelectedPid = null;
let currentTab = 'apps';
let currentPage = 1;
let pageSize = 100;
let sortColumn = null;
let sortDirection = 'asc';
let scanHistory = []; // Store scan sessions

// Tauri invoke helper with timeout
function getInvoke() {
	return new Promise((resolve, reject) => {
		const timeout = setTimeout(() => reject(new Error('Tauri init timeout')), 5000);
		const check = () => {
			const w = window.__TAURI__;
			const invoke = w?.invoke || w?.tauri?.invoke || w?.core?.invoke;
			if (invoke) {
				clearTimeout(timeout);
				resolve(invoke);
			} else {
				setTimeout(check, 50);
			}
		};
		check();
	});
}

// Status updates
function setStatus(msg) {
	document.getElementById('statusText').textContent = msg;
}

// Progress bar
function showProgress(title, percent, text) {
	const overlay = document.getElementById('progressOverlay');
	const bar = document.getElementById('progressBar');
	const titleEl = document.getElementById('progressTitle');
	const textEl = document.getElementById('progressText');
	
	if (percent >= 100 || percent < 0) {
		overlay.classList.remove('active');
		return;
	}
	
	overlay.classList.add('active');
	titleEl.textContent = title || 'Processing...';
	bar.style.width = Math.max(0, Math.min(100, percent)) + '%';
	textEl.textContent = text || `${Math.round(percent)}%`;
}

function hideProgress() {
	document.getElementById('progressOverlay').classList.remove('active');
}

// Type conversion helpers
function getTypeSpec(type) {
	const specs = {
		'i8': { dt: 'i8', size: 1, kind: 'int' },
		'i16': { dt: 'i16', size: 2, kind: 'int' },
		'i32': { dt: 'i32', size: 4, kind: 'int' },
		'i64': { dt: 'i64', size: 8, kind: 'int' },
		'f32': { dt: 'f32', size: 4, kind: 'float' },
		'f64': { dt: 'f64', size: 8, kind: 'float' },
		'string': { dt: 'string', size: 64, kind: 'string' },
		'bytes': { dt: 'bytes', size: 16, kind: 'bytes' }
	};
	return specs[type] || specs['i32'];
}

function toBytes(val, spec) {
	if (spec.kind === 'bytes') {
		const parts = String(val).trim().split(/\s+/);
		return parts.map(p => p === '??' ? 0 : parseInt(p, 16));
	}
	if (spec.kind === 'string') {
		const enc = new TextEncoder();
		const bytes = enc.encode(String(val || ''));
		const buf = new Uint8Array(spec.size);
		buf.set(bytes.slice(0, spec.size));
		return Array.from(buf);
	}
	
	const dv = new DataView(new ArrayBuffer(spec.size));
	if (spec.kind === 'int') {
		const v = spec.size === 8 ? BigInt(val || '0') : parseInt(val || '0', 10);
		if (spec.size === 1) dv.setInt8(0, v);
		else if (spec.size === 2) dv.setInt16(0, v, true);
		else if (spec.size === 4) dv.setInt32(0, v, true);
		else if (spec.size === 8) {
			dv.setUint32(0, Number(v & 0xFFFFFFFFn), true);
			dv.setUint32(4, Number((v >> 32n) & 0xFFFFFFFFn), true);
		}
	} else if (spec.kind === 'float') {
		const v = parseFloat(val || '0');
		if (spec.size === 4) dv.setFloat32(0, v, true);
		else if (spec.size === 8) dv.setFloat64(0, v, true);
	}
	return Array.from(new Uint8Array(dv.buffer));
}

function fromBytes(bytes, spec) {
	const arr = Uint8Array.from(bytes);
	if (spec.kind === 'string') {
		const dec = new TextDecoder();
		return dec.decode(arr).replace(/\u0000+$/, '');
	}
	if (spec.kind === 'bytes') {
		return Array.from(arr).map(b => b.toString(16).padStart(2, '0').toUpperCase()).join(' ');
	}
	
	const dv = new DataView(arr.buffer, arr.byteOffset, arr.byteLength);
	if (spec.kind === 'int') {
		if (spec.size === 1) return dv.getInt8(0);
		if (spec.size === 2) return dv.getInt16(0, true);
		if (spec.size === 4) return dv.getInt32(0, true);
		if (spec.size === 8) {
			const lo = dv.getUint32(0, true);
			const hi = dv.getUint32(4, true);
			return (BigInt(hi) << 32n) | BigInt(lo);
		}
	} else if (spec.kind === 'float') {
		if (spec.size === 4) return dv.getFloat32(0, true).toFixed(6);
		if (spec.size === 8) return dv.getFloat64(0, true).toFixed(6);
	}
	return 0;
}

// Process modal functions
function openProcessModal() {
	document.getElementById('processModal').classList.add('active');
	loadProcessesForModal();
}

function closeProcessModal() {
	document.getElementById('processModal').classList.remove('active');
	modalSelectedPid = null;
}

async function loadProcessesForModal() {
	try {
		setStatus('Loading processes...');
		showProgress('Loading processes...', 0, 'Enumerating processes...');
		const invoke = await getInvoke();
		
		// Show progress while loading
		let progress = 0;
		const progressInterval = setInterval(() => {
			progress = Math.min(progress + 5, 85);
			showProgress('Loading processes...', progress, `Loading... ${progress}%`);
		}, 100);
		
		allProcesses = await invoke('list_processes');
		
		clearInterval(progressInterval);
		showProgress('Loading processes...', 100, 'Complete!');
		setTimeout(hideProgress, 200);
		
		renderModalProcessList();
		setStatus(`Loaded ${allProcesses.length} processes`);
	} catch (e) {
		hideProgress();
		setStatus('Error loading processes: ' + (e?.message || e));
	}
}

function isSystemProcess(p) {
	const sys = ['system', 'registry', 'smss.exe', 'csrss.exe', 'wininit.exe', 'services.exe', 'lsass.exe', 'svchost.exe', 'dwm.exe'];
	return p.pid < 1000 || sys.some(s => p.name.toLowerCase().includes(s));
}

function filterModalProcesses() {
	const search = document.getElementById('modalSearch').value.toLowerCase();
	let filtered = allProcesses.filter(p => 
		p.name.toLowerCase().includes(search) || 
		p.pid.toString().includes(search)
	);
	
	if (currentTab === 'apps') {
		filtered = filtered.filter(p => !isSystemProcess(p));
	} else if (currentTab === 'system') {
		filtered = filtered.filter(p => isSystemProcess(p));
	} else if (currentTab === 'all') {
		// Show all processes
	}
	
	return filtered;
}

function renderModalProcessList() {
	const filtered = filterModalProcesses();
	const el = document.getElementById('modalProcessList');
	
	if (!filtered.length) {
		el.innerHTML = '<div class="empty-state">No processes found</div>';
		return;
	}
	
	// Sort by PID for consistency
	filtered.sort((a, b) => a.pid - b.pid);
	
	el.innerHTML = filtered.map(p => {
		const pidStr = String(p.pid).padStart(8, '0');
		const displayName = `${pidStr}-${p.name}`;
		// Show icon if available, otherwise show placeholder
		let iconHtml = '<div class="process-list-icon-placeholder" style="background: #3e3e42; border-radius: 4px; width: 32px; height: 32px;"></div>';
		if (p.icon && p.icon.length > 100) {
			// Check if it's a valid base64 data URI
			if (p.icon.startsWith('data:image') || p.icon.startsWith('data:image/png')) {
				iconHtml = `<img src="${p.icon}" class="process-list-icon" alt="" style="border-radius: 4px; width: 32px; height: 32px; object-fit: contain;" onerror="console.error('Icon load error for ${p.name}'); this.style.display='none'; this.parentElement.innerHTML='<div class=\\'process-list-icon-placeholder\\' style=\\'background: #3e3e42; border-radius: 4px; width: 32px; height: 32px;\\'></div>';" />`;
			} else if (p.icon.length > 50) {
				// Might be base64 without data URI prefix, add it
				iconHtml = `<img src="data:image/png;base64,${p.icon}" class="process-list-icon" alt="" style="border-radius: 4px; width: 32px; height: 32px; object-fit: contain;" onerror="console.error('Icon load error for ${p.name}'); this.style.display='none'; this.parentElement.innerHTML='<div class=\\'process-list-icon-placeholder\\' style=\\'background: #3e3e42; border-radius: 4px; width: 32px; height: 32px;\\'></div>';" />`;
			}
		}
		
		return `
			<div class="process-list-item ${modalSelectedPid === p.pid ? 'selected' : ''}" 
			     data-pid="${p.pid}" 
			     onclick="selectModalProcess(${p.pid})"
			     ondblclick="attachSelectedProcess()">
				${iconHtml}
				<div class="process-list-info">
					<div class="process-list-name">${displayName}</div>
				</div>
			</div>
		`;
	}).join('');
}

async function loadProcessIcons() {
	// Removed for now - will implement full icon extraction later
}

function selectModalProcess(pid) {
	modalSelectedPid = pid;
	renderModalProcessList();
	document.getElementById('btnAttach').disabled = false;
}

async function attachSelectedProcess() {
	if (!modalSelectedPid) return;
	
	try {
		const process = allProcesses.find(p => p.pid === modalSelectedPid);
		if (!process) return;
		
		setStatus(`Attaching to ${process.name} (${process.pid})...`);
		const invoke = await getInvoke();
		await invoke('attach_process', { pid: process.pid });
		
		selectedPid = process.pid;
		
		// Update process display
		const display = document.getElementById('processDisplay');
		const iconEl = display.querySelector('.process-icon');
		if (process.icon && (process.icon.startsWith('data:image') || process.icon.startsWith('data:image/png'))) {
			iconEl.innerHTML = `<img src="${process.icon}" style="width: 24px; height: 24px; object-fit: contain; border-radius: 4px;" alt="" onerror="this.style.display='none'; this.parentElement.innerHTML='<span style=\\'font-size: 20px;\\'>${isSystemProcess(process) ? '⚙️' : '📱'}</span>';" />`;
		} else {
			iconEl.innerHTML = `<span style="font-size: 20px;">${isSystemProcess(process) ? '⚙️' : '📱'}</span>`;
		}
		display.querySelector('.process-name').textContent = process.name;
		display.querySelector('.process-pid').textContent = `PID: ${process.pid}`;
		
		closeProcessModal();
		resetScan();
		setStatus(`Attached to ${process.name} (${process.pid})`);
	} catch (e) {
		setStatus('Attach error: ' + (e?.message || e));
	}
}

// Scan type change handler
function onScanTypeChange() {
	const type = document.getElementById('scanType').value;
	const valueGroup = document.getElementById('valueGroup');
	const rangeGroup = document.getElementById('rangeGroup');
	
	const noValueTypes = ['unknown', 'changed', 'unchanged', 'increased', 'decreased'];
	valueGroup.style.display = noValueTypes.includes(type) ? 'none' : 'block';
	rangeGroup.style.display = type === 'between' ? 'block' : 'none';
}

// Scan functions
async function performFirstScan() {
	if (!selectedPid) {
		alert('Please select a process first');
		return;
	}
	
	const type = document.getElementById('scanType').value;
	const valueType = document.getElementById('valueType').value;
	const spec = getTypeSpec(valueType);
	const value = document.getElementById('scanValue').value;
	const writableOnly = document.getElementById('optWritable').checked;
	const fastScan = document.getElementById('optFastScan').checked;
	
	if ((type === 'exact' || type === 'bigger' || type === 'smaller') && !value) {
		alert('Please enter a value to search');
		return;
	}
	
	let rangeMin = null;
	let rangeMax = null;
	if (type === 'between') {
		rangeMin = parseFloat(document.getElementById('rangeMin').value);
		rangeMax = parseFloat(document.getElementById('rangeMax').value);
		if (isNaN(rangeMin) || isNaN(rangeMax)) {
			alert('Please enter valid min and max values');
			return;
		}
	}
	
	try {
		setStatus('Scanning memory...');
		showProgress('Scanning memory...', 0, 'Initializing...');
		
		const invoke = await getInvoke();
		
		// Simulate progress (since we can't get real progress from Rust yet)
		let progress = 0;
		const progressInterval = setInterval(() => {
			progress = Math.min(progress + 2, 90);
			showProgress('Scanning memory...', progress, `Scanning... ${progress}%`);
		}, 100);
		
		const results = await invoke('scan_memory', {
			scanType: type,
			value: (type === 'unknown' || type === 'changed' || type === 'unchanged') ? null : (value || null),
			dataType: spec.dt,
			rangeMin: rangeMin,
			rangeMax: rangeMax,
			writableOnly: writableOnly,
			aligned: fastScan
		});
		
		clearInterval(progressInterval);
		showProgress('Scanning memory...', 100, 'Complete!');
		setTimeout(hideProgress, 300);
		
		previousScanResults = scanResults = filteredResults = results;
		currentPage = 1;
		renderResults(results);
		
		document.getElementById('btnFirstScan').disabled = true;
		document.getElementById('btnNextScan').disabled = false;
		document.getElementById('btnNewScan').disabled = false;
		
		setStatus(`Found ${results.length} addresses`);
	} catch (e) {
		hideProgress();
		setStatus('Scan error: ' + (e?.message || e));
		console.error('Scan error:', e);
	}
}

async function performNextScan() {
	if (!previousScanResults || !previousScanResults.length) {
		alert('No previous scan results');
		return;
	}
	
	if (!selectedPid) {
		alert('No process attached');
		return;
	}
	
	const type = document.getElementById('scanType').value;
	const valueType = document.getElementById('valueType').value;
	const spec = getTypeSpec(valueType);
	const value = document.getElementById('scanValue').value;
	
	let rangeMin = null;
	let rangeMax = null;
	if (type === 'between') {
		rangeMin = parseFloat(document.getElementById('rangeMin').value);
		rangeMax = parseFloat(document.getElementById('rangeMax').value);
		if (isNaN(rangeMin) || isNaN(rangeMax)) {
			alert('Please enter valid min and max values');
			return;
		}
	}
	
	try {
		setStatus('Re-scanning memory...');
		showProgress('Re-scanning memory...', 0, 'Initializing...');
		
		const invoke = await getInvoke();
		
		let progress = 0;
		const progressInterval = setInterval(() => {
			progress = Math.min(progress + 3, 90);
			showProgress('Re-scanning memory...', progress, `Re-scanning... ${progress}%`);
		}, 80);
		
		const results = await invoke('rescan_memory', {
			scanType: type,
			value: (type === 'unknown' || type === 'changed' || type === 'unchanged') ? null : (value || null),
			dataType: spec.dt,
			rangeMin: rangeMin,
			rangeMax: rangeMax
		});
		
		clearInterval(progressInterval);
		showProgress('Re-scanning memory...', 100, 'Complete!');
		setTimeout(hideProgress, 300);
		
		previousScanResults = scanResults = filteredResults = results;
		currentPage = 1;
		renderResults(results);
		setStatus(`Found ${results.length} addresses`);
	} catch (e) {
		hideProgress();
		setStatus('Scan error: ' + (e?.message || e));
		console.error('Rescan error:', e);
	}
}

function resetScan() {
	scanResults = [];
	filteredResults = [];
	previousScanResults = null;
	currentPage = 1;
	sortColumn = null;
	sortDirection = 'asc';
	
	document.getElementById('resultsBody').innerHTML = 
		'<tr><td colspan="4" class="empty-state">No scan results yet. Select a process and perform a scan.</td></tr>';
	document.getElementById('resultsCount').textContent = '';
	document.getElementById('pagination').style.display = 'none';
	document.getElementById('btnFirstScan').disabled = false;
	document.getElementById('btnNextScan').disabled = true;
	document.getElementById('btnNewScan').disabled = true;
	document.getElementById('filterInput').value = '';
	
	// Reset sort indicators
	document.querySelectorAll('.sortable').forEach(th => {
		th.classList.remove('asc', 'desc');
	});
	
	setStatus('Scan reset. Ready for new scan.');
}

function renderResults(list) {
	const tbody = document.getElementById('resultsBody');
	const count = document.getElementById('resultsCount');
	const pagination = document.getElementById('pagination');
	
	if (!list.length) {
		tbody.innerHTML = '<tr><td colspan="4" class="empty-state">No addresses found</td></tr>';
		count.textContent = '';
		pagination.style.display = 'none';
		return;
	}
	
	// Sort if needed
	let sorted = [...list];
	if (sortColumn) {
		sorted.sort((a, b) => {
			let valA, valB;
			if (sortColumn === 'index') {
				// For index, use original position in list
				valA = list.indexOf(a);
				valB = list.indexOf(b);
			} else if (sortColumn === 'address') {
				// Parse hex addresses
				valA = parseInt(a.address.replace('0x', ''), 16) || 0;
				valB = parseInt(b.address.replace('0x', ''), 16) || 0;
			} else if (sortColumn === 'value') {
				// Try to parse as number, fallback to string comparison
				const numA = parseFloat(a.value);
				const numB = parseFloat(b.value);
				if (!isNaN(numA) && !isNaN(numB)) {
					valA = numA;
					valB = numB;
				} else {
					valA = String(a.value).toLowerCase();
					valB = String(b.value).toLowerCase();
				}
			} else {
				valA = a[sortColumn];
				valB = b[sortColumn];
			}
			
			if (valA < valB) return sortDirection === 'asc' ? -1 : 1;
			if (valA > valB) return sortDirection === 'asc' ? 1 : -1;
			return 0;
		});
	}
	
	// Pagination
	const totalPages = Math.ceil(sorted.length / pageSize);
	const startIdx = (currentPage - 1) * pageSize;
	const endIdx = Math.min(startIdx + pageSize, sorted.length);
	const pageData = sorted.slice(startIdx, endIdx);
	
	tbody.innerHTML = pageData.map((r, i) => {
		const globalIdx = startIdx + i;
		// Escape quotes for onclick handlers
		const safeAddress = String(r.address).replace(/'/g, "\\'").replace(/"/g, '&quot;');
		const safeValue = String(r.value).replace(/'/g, "\\'").replace(/"/g, '&quot;');
		return `
			<tr>
				<td>${globalIdx + 1}</td>
				<td><code>${r.address}</code></td>
				<td><input type="text" value="${safeValue}" onkeypress="if(event.key==='Enter') updateValue('${safeAddress}', this.value, ${globalIdx})" /></td>
				<td>
					<button class="btn btn-secondary" style="padding: 4px 12px; font-size: 11px; margin-right: 4px;" onclick="addToAddressList('${safeAddress}', '${safeValue}')">Add</button>
					<button class="btn btn-secondary" style="padding: 4px 12px; font-size: 11px;" onclick="openHexEditor('${safeAddress}')" title="Open in Hex Editor">Hex</button>
				</td>
			</tr>
		`;
	}).join('');
	
	// Update pagination
	if (totalPages > 1) {
		pagination.style.display = 'flex';
		document.getElementById('pageInfo').textContent = `Page ${currentPage} of ${totalPages} (${sorted.length} total)`;
		document.getElementById('btnPrev').disabled = currentPage === 1;
		document.getElementById('btnNext').disabled = currentPage === totalPages;
	} else {
		pagination.style.display = 'none';
	}
	
	count.textContent = `(${list.length} found)`;
}

function changePage(delta) {
	const totalPages = Math.ceil(filteredResults.length / pageSize);
	const newPage = Math.max(1, Math.min(totalPages, currentPage + delta));
	if (newPage !== currentPage) {
		currentPage = newPage;
		renderResults(filteredResults);
		// Scroll to top of results
		document.querySelector('.results-panel .panel-content').scrollTop = 0;
	}
}

function sortResults(column) {
	if (sortColumn === column) {
		sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
	} else {
		sortColumn = column;
		sortDirection = 'asc';
	}
	
	// Update UI
	document.querySelectorAll('.sortable').forEach(th => {
		th.classList.remove('asc', 'desc');
		if (th.dataset.sort === column) {
			th.classList.add(sortDirection);
		}
	});
	
	currentPage = 1;
	renderResults(filteredResults);
}

function filterResults() {
	const query = document.getElementById('filterInput').value.toLowerCase();
	if (!query) {
		filteredResults = scanResults;
	} else {
		filteredResults = scanResults.filter(r => 
			r.address.toLowerCase().includes(query) || 
			String(r.value).toLowerCase().includes(query)
		);
	}
	currentPage = 1; // Reset to first page when filtering
	renderResults(filteredResults);
}

function clearFilter() {
	document.getElementById('filterInput').value = '';
	filteredResults = scanResults;
	currentPage = 1; // Reset to first page when clearing filter
	renderResults(filteredResults);
}

async function updateValue(address, newValue, index) {
	if (!selectedPid) return;
	
	try {
		const spec = getTypeSpec(document.getElementById('valueType').value);
		const data = toBytes(newValue, spec);
		const invoke = await getInvoke();
		
		await invoke('write_memory', {
			address: address,
			data: data
		});
		
		scanResults[index].value = newValue;
		setStatus(`Updated ${address} to ${newValue}`);
	} catch (e) {
		alert('Failed to update: ' + (e?.message || e));
	}
}

// Address list functions
function addToAddressList(address, value) {
	const type = document.getElementById('valueType').value;
	addressList.push({
		id: Date.now(),
		active: true,
		freeze: false,
		description: `Address ${addressList.length + 1}`,
		address: address,
		type: type,
		value: value
	});
	renderAddressList();
	setStatus(`Added ${address} to address list`);
}

function renderAddressList() {
	const tbody = document.getElementById('addressTableBody');
	
	if (!addressList.length) {
		tbody.innerHTML = '<tr><td colspan="7" class="empty-state">No addresses added yet. Add addresses from scan results to watch and freeze values.</td></tr>';
		return;
	}
	
	tbody.innerHTML = addressList.map(e => {
		const safeAddress = String(e.address).replace(/'/g, "\\'").replace(/"/g, '&quot;');
		const safeDesc = String(e.description).replace(/'/g, "\\'").replace(/"/g, '&quot;');
		const safeValue = String(e.value).replace(/'/g, "\\'").replace(/"/g, '&quot;');
		return `
		<tr>
			<td><input type="checkbox" ${e.active ? 'checked' : ''} onchange="toggleAddressActive(${e.id}, this.checked)" /></td>
			<td><input type="checkbox" ${e.freeze ? 'checked' : ''} onchange="toggleAddressFreeze(${e.id}, this.checked)" /></td>
			<td><input type="text" value="${safeDesc}" onchange="updateAddressDescription(${e.id}, this.value)" /></td>
			<td><code>${e.address}</code></td>
			<td>${e.type}</td>
			<td><input type="text" value="${safeValue}" onkeypress="if(event.key==='Enter') updateAddressValue(${e.id}, this.value)" /></td>
			<td>
				<button class="btn btn-secondary" style="padding: 4px 12px; font-size: 11px; margin-right: 4px;" onclick="openHexEditor('${safeAddress}')" title="Open in Hex Editor">Hex</button>
				<button class="btn btn-danger" style="padding: 4px 12px; font-size: 11px;" onclick="removeFromAddressList(${e.id})">Remove</button>
			</td>
		</tr>
		`;
	}).join('');
}

function toggleAddressActive(id, active) {
	const addr = addressList.find(a => a.id === id);
	if (addr) addr.active = active;
}

function toggleAddressFreeze(id, freeze) {
	const addr = addressList.find(a => a.id === id);
	if (addr) addr.freeze = freeze;
}

function updateAddressDescription(id, description) {
	const addr = addressList.find(a => a.id === id);
	if (addr) addr.description = description;
}

async function updateAddressValue(id, newValue) {
	const addr = addressList.find(a => a.id === id);
	if (!addr || !selectedPid) return;
	
	try {
		const spec = getTypeSpec(addr.type);
		const data = toBytes(newValue, spec);
		const invoke = await getInvoke();
		
		await invoke('write_memory', {
			address: addr.address,
			data: data
		});
		
		addr.value = newValue;
		renderAddressList();
		setStatus(`Updated ${addr.address} to ${newValue}`);
	} catch (e) {
		alert('Failed: ' + (e?.message || e));
		setStatus('Update error: ' + (e?.message || e));
	}
}

function removeFromAddressList(id) {
	addressList = addressList.filter(a => a.id !== id);
	renderAddressList();
	setStatus('Address removed');
}

// Freeze loop
function startFreezeLoop() {
	setInterval(async () => {
		if (!selectedPid) return;
		
		const frozen = addressList.filter(a => a.active && a.freeze);
		if (!frozen.length) return;
		
		try {
			const invoke = await getInvoke();
			for (const addr of frozen) {
				try {
					const spec = getTypeSpec(addr.type);
					const data = toBytes(addr.value, spec);
					await invoke('write_memory', {
						address: addr.address,
						data: data
					});
				} catch {}
			}
		} catch {}
	}, 100);
}

// Event listeners
document.addEventListener('DOMContentLoaded', async () => {
	// Scan controls
	document.getElementById('scanType').addEventListener('change', onScanTypeChange);
	document.getElementById('btnFirstScan').addEventListener('click', performFirstScan);
	document.getElementById('btnNextScan').addEventListener('click', performNextScan);
	document.getElementById('btnNewScan').addEventListener('click', resetScan);
	
	// Modal tabs
	document.querySelectorAll('.modal-tab').forEach(tab => {
		tab.addEventListener('click', () => {
			document.querySelectorAll('.modal-tab').forEach(t => t.classList.remove('active'));
			tab.classList.add('active');
			currentTab = tab.dataset.tab;
			renderModalProcessList();
		});
	});
	
	// Modal search
	document.getElementById('modalSearch').addEventListener('input', renderModalProcessList);
	
	// Sortable columns
	document.querySelectorAll('.sortable').forEach(th => {
		th.addEventListener('click', () => {
			sortResults(th.dataset.sort);
		});
	});
	
	// Start freeze loop
	startFreezeLoop();
	
	// Get version
	try {
		const invoke = await getInvoke();
		const version = await invoke('get_version');
		document.getElementById('versionText').textContent = version;
	} catch {
		document.getElementById('versionText').textContent = 'v0.0.0';
	}
	
	setStatus('Ready - Select a process to begin');
});

// Hex Editor Functions
let currentHexAddress = null;
let currentHexData = null;

function openHexEditor(address) {
	const panel = document.getElementById('hexEditorPanel');
	const content = document.querySelector('.content');
	panel.style.display = 'flex';
	content.classList.add('with-hex');
	
	if (address) {
		document.getElementById('hexAddressInput').value = address;
		loadHexView();
	}
}

function closeHexEditor() {
	const panel = document.getElementById('hexEditorPanel');
	const content = document.querySelector('.content');
	panel.style.display = 'none';
	content.classList.remove('with-hex');
	currentHexAddress = null;
	currentHexData = null;
}

async function loadHexView() {
	if (!selectedPid) {
		alert('Please select a process first');
		return;
	}
	
	const addressInput = document.getElementById('hexAddressInput').value.trim();
	const sizeInput = parseInt(document.getElementById('hexSizeInput').value) || 256;
	
	if (!addressInput) {
		alert('Please enter an address');
		return;
	}
	
	try {
		setStatus('Loading memory...');
		const invoke = await getInvoke();
		
		const data = await invoke('read_memory', {
			pid: selectedPid,
			address: addressInput,
			size: sizeInput
		});
		
		currentHexAddress = addressInput;
		currentHexData = data;
		renderHexView(data, addressInput);
		setStatus(`Loaded ${data.length} bytes from ${addressInput}`);
	} catch (e) {
		setStatus('Error loading memory: ' + (e?.message || e));
		alert('Failed to load memory: ' + (e?.message || e));
	}
}

async function refreshHexView() {
	if (currentHexAddress) {
		await loadHexView();
	}
}

function renderHexView(data, startAddress) {
	const container = document.getElementById('hexViewContainer');
	
	// Parse start address
	const baseAddr = startAddress.startsWith('0x') 
		? parseInt(startAddress.slice(2), 16) 
		: parseInt(startAddress, 16);
	
	let html = '<div class="hex-view">';
	const bytesPerLine = 16;
	
	for (let i = 0; i < data.length; i += bytesPerLine) {
		const lineAddr = baseAddr + i;
		const lineBytes = data.slice(i, i + bytesPerLine);
		
		html += '<div class="hex-line">';
		html += `<div class="hex-address">0x${lineAddr.toString(16).toUpperCase().padStart(8, '0')}</div>`;
		html += '<div class="hex-bytes">';
		
		for (let j = 0; j < bytesPerLine; j++) {
			if (j < lineBytes.length) {
				const byte = lineBytes[j];
				const byteAddr = lineAddr + j;
				html += `<span class="hex-byte" onclick="editHexByte(${byteAddr}, ${byte})" title="Click to edit">${byte.toString(16).toUpperCase().padStart(2, '0')}</span>`;
			} else {
				html += '<span class="hex-byte" style="opacity: 0.3;">--</span>';
			}
		}
		
		html += '</div>';
		html += '<div class="hex-ascii">';
		
		for (let j = 0; j < bytesPerLine; j++) {
			if (j < lineBytes.length) {
				const byte = lineBytes[j];
				const char = (byte >= 32 && byte < 127) ? String.fromCharCode(byte) : '.';
				const className = (byte >= 32 && byte < 127) ? '' : 'non-printable';
				html += `<span class="hex-ascii-char ${className}">${char}</span>`;
			} else {
				html += '<span class="hex-ascii-char" style="opacity: 0.3;"> </span>';
			}
		}
		
		html += '</div>';
		html += '</div>';
	}
	
	html += '</div>';
	container.innerHTML = html;
}

function editHexByte(address, currentValue) {
	const newValue = prompt(`Edit byte at 0x${address.toString(16).toUpperCase()}\nCurrent: 0x${currentValue.toString(16).toUpperCase().padStart(2, '0')} (${currentValue})\nEnter new value (0-255 or hex):`, currentValue.toString(16).toUpperCase());
	
	if (newValue === null) return;
	
	let byteValue;
	if (newValue.startsWith('0x') || newValue.startsWith('0X')) {
		byteValue = parseInt(newValue.slice(2), 16);
	} else {
		byteValue = parseInt(newValue, 10);
	}
	
	if (isNaN(byteValue) || byteValue < 0 || byteValue > 255) {
		alert('Invalid value. Must be between 0 and 255.');
		return;
	}
	
	writeHexByte(address, byteValue);
}

async function writeHexByte(address, value) {
	if (!selectedPid) {
		alert('No process attached');
		return;
	}
	
	try {
		setStatus(`Writing byte to 0x${address.toString(16).toUpperCase()}...`);
		const invoke = await getInvoke();
		
		await invoke('write_memory', {
			address: `0x${address.toString(16)}`,
			data: [value]
		});
		
		// Refresh the hex view
		await refreshHexView();
		setStatus(`Byte written successfully`);
	} catch (e) {
		setStatus('Error writing memory: ' + (e?.message || e));
		alert('Failed to write memory: ' + (e?.message || e));
	}
}

// Scan Session Management
function saveScanSession() {
	if (!scanResults.length) {
		alert('No scan results to save');
		return;
	}
	
	const sessionName = prompt('Enter a name for this scan session:', `Scan_${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}`);
	if (!sessionName) return;
	
	const session = {
		name: sessionName,
		timestamp: new Date().toISOString(),
		pid: selectedPid,
		processName: allProcesses.find(p => p.pid === selectedPid)?.name || 'Unknown',
		scanResults: scanResults,
		addressList: addressList,
		scanConfig: {
			type: document.getElementById('scanType').value,
			valueType: document.getElementById('valueType').value,
			value: document.getElementById('scanValue').value,
			writableOnly: document.getElementById('optWritable').checked,
			fastScan: document.getElementById('optFastScan').checked
		}
	};
	
	try {
		// Load existing sessions
		const existing = JSON.parse(localStorage.getItem('inspectra_scans') || '[]');
		existing.push(session);
		localStorage.setItem('inspectra_scans', JSON.stringify(existing));
		setStatus(`Scan session "${sessionName}" saved successfully`);
		alert(`Scan session "${sessionName}" saved successfully!`);
	} catch (e) {
		setStatus('Error saving scan: ' + (e?.message || e));
		alert('Failed to save scan session: ' + (e?.message || e));
	}
}

function loadScanSession() {
	try {
		const sessions = JSON.parse(localStorage.getItem('inspectra_scans') || '[]');
		
		if (!sessions.length) {
			alert('No saved scan sessions found');
			return;
		}
		
		// Create selection dialog
		const sessionList = sessions.map((s, i) => 
			`${i + 1}. ${s.name} (${s.processName}, ${s.scanResults.length} results, ${new Date(s.timestamp).toLocaleString()})`
		).join('\n');
		
		const choice = prompt(`Select a scan session to load (1-${sessions.length}):\n\n${sessionList}\n\nEnter number:`, '1');
		if (!choice) return;
		
		const index = parseInt(choice) - 1;
		if (isNaN(index) || index < 0 || index >= sessions.length) {
			alert('Invalid selection');
			return;
		}
		
		const session = sessions[index];
		
		// Check if process is still available
		if (selectedPid !== session.pid) {
			const confirmLoad = confirm(`This scan was from process PID ${session.pid} (${session.processName}).\nCurrent process is PID ${selectedPid}.\nLoad anyway?`);
			if (!confirmLoad) return;
		}
		
		// Restore scan results
		scanResults = session.scanResults;
		filteredResults = scanResults;
		addressList = session.addressList || [];
		
		// Restore scan config if available
		if (session.scanConfig) {
			document.getElementById('scanType').value = session.scanConfig.type || 'exact';
			document.getElementById('valueType').value = session.scanConfig.valueType || 'i32';
			document.getElementById('scanValue').value = session.scanConfig.value || '';
			document.getElementById('optWritable').checked = session.scanConfig.writableOnly || false;
			document.getElementById('optFastScan').checked = session.scanConfig.fastScan || false;
			onScanTypeChange();
		}
		
		// Update UI
		currentPage = 1;
		renderResults(scanResults);
		renderAddressList();
		
		document.getElementById('btnFirstScan').disabled = true;
		document.getElementById('btnNextScan').disabled = false;
		document.getElementById('btnNewScan').disabled = false;
		
		setStatus(`Loaded scan session "${session.name}" with ${scanResults.length} results`);
	} catch (e) {
		setStatus('Error loading scan: ' + (e?.message || e));
		alert('Failed to load scan session: ' + (e?.message || e));
	}
}
