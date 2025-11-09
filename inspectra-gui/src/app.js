// Global State
let allProcesses = [];
let selectedPid = null;
let scanResults = [];
let filteredResults = [];
let addressList = [];
let previousScanResults = null;
let modalSelectedPid = null;
let currentTab = 'apps';

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
		const invoke = await getInvoke();
		allProcesses = await invoke('list_processes');
		renderModalProcessList();
		setStatus(`Loaded ${allProcesses.length} processes`);
	} catch (e) {
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
		
		return `
			<div class="process-list-item ${modalSelectedPid === p.pid ? 'selected' : ''}" 
			     data-pid="${p.pid}" 
			     onclick="selectModalProcess(${p.pid})"
			     ondblclick="attachSelectedProcess()">
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
		display.querySelector('.process-icon').textContent = isSystemProcess(process) ? '⚙️' : '📱';
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
	
	if (type === 'exact' && !value) {
		alert('Please enter a value to search');
		return;
	}
	
	if (type === 'between') {
		const min = document.getElementById('rangeMin').value;
		const max = document.getElementById('rangeMax').value;
		if (!min || !max) {
			alert('Please enter min and max values');
			return;
		}
	}
	
	try {
		setStatus('Scanning memory...');
		const invoke = await getInvoke();
		
		// For now, only exact value is supported by backend
		if (type !== 'exact') {
			setStatus('Only Exact Value scan is fully supported. Using exact value scan.');
		}
		
		const results = await invoke('scan_memory', {
			value: String(value || '0'),
			data_type: spec.dt
		});
		
		previousScanResults = scanResults = filteredResults = results;
		renderResults(results);
		
		document.getElementById('btnFirstScan').disabled = true;
		document.getElementById('btnNextScan').disabled = false;
		document.getElementById('btnNewScan').disabled = false;
		
		setStatus(`Found ${results.length} addresses`);
	} catch (e) {
		setStatus('Scan error: ' + (e?.message || e));
	}
}

async function performNextScan() {
	if (!previousScanResults || !previousScanResults.length) {
		alert('No previous scan results');
		return;
	}
	
	const type = document.getElementById('scanType').value;
	const valueType = document.getElementById('valueType').value;
	const spec = getTypeSpec(valueType);
	const value = document.getElementById('scanValue').value;
	
	try {
		setStatus('Re-scanning memory...');
		const invoke = await getInvoke();
		const next = [];
		
		for (const r of previousScanResults) {
			try {
				const bytes = await invoke('read_memory', {
					pid: selectedPid,
					address: r.address,
					size: spec.size
				});
				
				const current = fromBytes(bytes, spec);
				let match = false;
				
				if (type === 'exact') {
					match = String(current) === String(value);
				} else if (type === 'bigger') {
					match = parseFloat(current) > parseFloat(value);
				} else if (type === 'smaller') {
					match = parseFloat(current) < parseFloat(value);
				} else if (type === 'between') {
					const min = parseFloat(document.getElementById('rangeMin').value);
					const max = parseFloat(document.getElementById('rangeMax').value);
					const v = parseFloat(current);
					match = v >= min && v <= max;
				} else if (type === 'changed') {
					match = String(current) !== String(r.value);
				} else if (type === 'unchanged') {
					match = String(current) === String(r.value);
				} else if (type === 'increased') {
					match = parseFloat(current) > parseFloat(r.value);
				} else if (type === 'decreased') {
					match = parseFloat(current) < parseFloat(r.value);
				}
				
				if (match) {
					next.push({ address: r.address, value: current });
				}
			} catch {}
		}
		
		previousScanResults = scanResults = filteredResults = next;
		renderResults(next);
		setStatus(`Found ${next.length} addresses`);
	} catch (e) {
		setStatus('Scan error: ' + (e?.message || e));
	}
}

function resetScan() {
	scanResults = [];
	filteredResults = [];
	previousScanResults = null;
	
	document.getElementById('resultsBody').innerHTML = 
		'<tr><td colspan="4" class="empty-state">No scan results yet. Select a process and perform a scan.</td></tr>';
	document.getElementById('resultsCount').textContent = '';
	document.getElementById('btnFirstScan').disabled = false;
	document.getElementById('btnNextScan').disabled = true;
	document.getElementById('btnNewScan').disabled = true;
	document.getElementById('filterInput').value = '';
	
	setStatus('Scan reset');
}

function renderResults(list) {
	const tbody = document.getElementById('resultsBody');
	const count = document.getElementById('resultsCount');
	
	if (!list.length) {
		tbody.innerHTML = '<tr><td colspan="4" class="empty-state">No addresses found</td></tr>';
		count.textContent = '';
		return;
	}
	
	const limit = 500;
	const displayed = Math.min(list.length, limit);
	
	tbody.innerHTML = list.slice(0, limit).map((r, i) => `
		<tr>
			<td>${i + 1}</td>
			<td><code>${r.address}</code></td>
			<td><input type="text" value="${r.value}" onkeypress="if(event.key==='Enter') updateValue('${r.address}', this.value, ${i})" /></td>
			<td><button class="btn btn-secondary" style="padding: 4px 12px; font-size: 11px;" onclick="addToAddressList('${r.address}', '${r.value}')">Add</button></td>
		</tr>
	`).join('');
	
	if (list.length > limit) {
		tbody.innerHTML += `<tr><td colspan="4" class="empty-state">Showing ${displayed} of ${list.length} results</td></tr>`;
	}
	
	count.textContent = `(${list.length} found)`;
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
	renderResults(filteredResults);
}

function clearFilter() {
	document.getElementById('filterInput').value = '';
	filteredResults = scanResults;
	renderResults(filteredResults);
}

async function updateValue(address, newValue, index) {
	if (!selectedPid) return;
	
	try {
		const spec = getTypeSpec(document.getElementById('valueType').value);
		const data = toBytes(newValue, spec);
		const invoke = await getInvoke();
		
		await invoke('write_memory', {
			pid: selectedPid,
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
	
	tbody.innerHTML = addressList.map(e => `
		<tr>
			<td><input type="checkbox" ${e.active ? 'checked' : ''} onchange="toggleAddressActive(${e.id}, this.checked)" /></td>
			<td><input type="checkbox" ${e.freeze ? 'checked' : ''} onchange="toggleAddressFreeze(${e.id}, this.checked)" /></td>
			<td><input type="text" value="${e.description}" onchange="updateAddressDescription(${e.id}, this.value)" /></td>
			<td><code>${e.address}</code></td>
			<td>${e.type}</td>
			<td><input type="text" value="${e.value}" onkeypress="if(event.key==='Enter') updateAddressValue(${e.id}, this.value)" /></td>
			<td><button class="btn btn-danger" style="padding: 4px 12px; font-size: 11px;" onclick="removeFromAddressList(${e.id})">Remove</button></td>
		</tr>
	`).join('');
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
			pid: selectedPid,
			address: addr.address,
			data: data
		});
		
		addr.value = newValue;
		renderAddressList();
		setStatus('Value updated');
	} catch (e) {
		alert('Failed: ' + (e?.message || e));
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
						pid: selectedPid,
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
