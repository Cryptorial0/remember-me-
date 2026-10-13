const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const $ = (id) => document.getElementById(id);

const pad = (n) => String(n).padStart(2, '0');
const toInput = (d) =>
  `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;

let editingId = null;

function setDefaultTime() {
  $('when').value = toInput(new Date(Date.now() + 10 * 60000));
}

function resetForm() {
  editingId = null;
  $('title').value = '';
  $('repeat').value = 'none';
  $('save').textContent = 'Add reminder';
  $('cancel').hidden = true;
  setDefaultTime();
}

function startEdit(r) {
  editingId = r.id;
  $('title').value = r.title;
  $('when').value = toInput(new Date(r.due * 1000));
  $('repeat').value = r.repeat;
  $('save').textContent = 'Save changes';
  $('cancel').hidden = false;
  window.scrollTo(0, 0);
  $('title').focus();
}

document.querySelectorAll('.chips button').forEach((b) =>
  b.addEventListener('click', () => {
    const m = b.dataset.m;
    const d = new Date();
    if (m === 'tom') { d.setDate(d.getDate() + 1); d.setHours(9, 0, 0, 0); }
    else d.setMinutes(d.getMinutes() + Number(m));
    $('when').value = toInput(d);
  })
);

$('cancel').onclick = resetForm;

$('f').addEventListener('submit', async (e) => {
  e.preventDefault();
  const due = Math.floor(new Date($('when').value).getTime() / 1000);
  const args = { title: $('title').value, due, repeat: $('repeat').value };
  if (editingId !== null) await invoke('update_reminder', { id: editingId, ...args });
  else await invoke('add_reminder', args);
  resetForm();
  $('title').focus();
  refresh();
});

function btn(label, fn) {
  const b = document.createElement('button');
  b.textContent = label;
  b.onclick = async () => { await fn(); refresh(); };
  return b;
}

async function refresh() {
  const items = await invoke('list_reminders');
  const ul = $('list');
  ul.innerHTML = '';
  const nowS = Date.now() / 1000;
  for (const r of items) {
    const li = document.createElement('li');
    if (r.done) li.classList.add('done');
    if (!r.done && r.due < nowS) li.classList.add('over');
    const t = document.createElement('div'); t.className = 't'; t.textContent = r.title;
    const d = document.createElement('div'); d.className = 'd';
    d.textContent = new Date(r.due * 1000).toLocaleString() + (r.repeat !== 'none' ? '  ·  ' + r.repeat : '');
    const bs = document.createElement('div'); bs.className = 'btns';
    const edit = document.createElement('button');
    edit.textContent = 'Edit';
    edit.onclick = () => startEdit(r);
    bs.append(
      btn(r.done ? 'Undo' : 'Done', () => invoke('toggle_done', { id: r.id })),
      edit,
      btn('Snooze 10m', () => invoke('snooze', { id: r.id, minutes: 10 })),
      btn('Snooze 1h', () => invoke('snooze', { id: r.id, minutes: 60 })),
      btn('Delete', () => invoke('delete_reminder', { id: r.id }))
    );
    li.append(t, d, bs);
    ul.append(li);
  }
}

// ---------- settings ----------
const sSound = $('s-sound'), sVol = $('s-vol'), sRep = $('s-rep'), sFile = $('s-file'),
      sName = $('s-fname'), sAuto = $('s-auto'), sHot = $('s-hotkey');

async function loadSettings() {
  const s = await invoke('get_settings');
  sSound.value = s.sound || 'chime';
  sVol.value = s.volume || '0.6';
  sRep.value = s.repeats ?? '0';
  sName.textContent = s.custom_name || 'none';
  sAuto.checked = s.autostart !== '0';
  sHot.value = s.hotkey || 'Alt+X';
}
sSound.onchange = () => invoke('set_setting', { key: 'sound', value: sSound.value });
sVol.onchange = () => invoke('set_setting', { key: 'volume', value: sVol.value });
sRep.onchange = () => {
  const v = parseInt(sRep.value);
  const n = isNaN(v) ? 0 : Math.max(0, Math.min(10, v));
  sRep.value = n;
  invoke('set_setting', { key: 'repeats', value: String(n) });
};
sAuto.onchange = () => invoke('set_autostart', { enabled: sAuto.checked }).catch((e) => alert(e));
sFile.onchange = async () => {
  const f = sFile.files[0];
  if (!f) return;
  if (f.size > 2 * 1024 * 1024) { alert('File too big (max 2 MB).'); sFile.value = ''; return; }
  const url = await new Promise((res) => { const fr = new FileReader(); fr.onload = () => res(fr.result); fr.readAsDataURL(f); });
  await invoke('set_setting', { key: 'custom_sound', value: url });
  await invoke('set_setting', { key: 'custom_name', value: f.name });
  await invoke('set_setting', { key: 'sound', value: 'custom' });
  sSound.value = 'custom';
  sName.textContent = f.name;
};
$('s-test').onclick = async () => {
  let url = null;
  if (sSound.value === 'custom') url = await invoke('get_custom_sound');
  Sound.play(sSound.value, parseFloat(sVol.value), url);
};
$('s-stop').onclick = () => Sound.stop();

// hotkey capture
function keyName(code) {
  if (/^Key[A-Z]$/.test(code)) return code.slice(3);
  if (/^Digit[0-9]$/.test(code)) return code.slice(5);
  if (/^F([1-9]|1[0-2])$/.test(code)) return code;
  const m = { Space: 'Space', ArrowUp: 'Up', ArrowDown: 'Down', ArrowLeft: 'Left', ArrowRight: 'Right' };
  return m[code] || null;
}
sHot.addEventListener('keydown', async (e) => {
  e.preventDefault();
  if (['Control', 'Alt', 'Shift', 'Meta'].includes(e.key)) return;
  const k = keyName(e.code);
  if (!k) { alert('Use a letter, number, F1-F12, Space or an arrow key.'); return; }
  if (!(e.ctrlKey || e.altKey)) { alert('Include Ctrl or Alt (for example Alt+X).'); return; }
  const mods = [];
  if (e.ctrlKey) mods.push('Ctrl');
  if (e.altKey) mods.push('Alt');
  if (e.shiftKey) mods.push('Shift');
  const combo = [...mods, k].join('+');
  try {
    await invoke('set_hotkey', { hotkey: combo });
    sHot.value = combo;
  } catch (err) {
    alert('Could not use ' + combo + ': ' + err);
  }
});

// ---------- quit confirmation ----------
listen('confirm-quit', () => { $('quit').hidden = false; });
$('q-no').onclick = () => { $('quit').hidden = true; };
$('q-yes').onclick = () => invoke('quit_app');

listen('quick-add', () => {
  if (editingId === null) setDefaultTime();
  $('title').focus();
  $('title').select();
  refresh();
});
listen('changed', refresh);

setDefaultTime();
refresh();
loadSettings();
setInterval(refresh, 30000);
