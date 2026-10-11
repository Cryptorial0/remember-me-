const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const $ = (id) => document.getElementById(id);

const pad = (n) => String(n).padStart(2, '0');
const toInput = (d) =>
  `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;

function setDefaultTime() {
  $('when').value = toInput(new Date(Date.now() + 10 * 60000));
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

$('f').addEventListener('submit', async (e) => {
  e.preventDefault();
  const due = Math.floor(new Date($('when').value).getTime() / 1000);
  await invoke('add_reminder', { title: $('title').value, due, repeat: $('repeat').value });
  $('title').value = '';
  $('repeat').value = 'none';
  setDefaultTime();
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
    bs.append(
      btn(r.done ? 'Undo' : 'Done', () => invoke('toggle_done', { id: r.id })),
      btn('Snooze 10m', () => invoke('snooze', { id: r.id, minutes: 10 })),
      btn('Snooze 1h', () => invoke('snooze', { id: r.id, minutes: 60 })),
      btn('Delete', () => invoke('delete_reminder', { id: r.id }))
    );
    li.append(t, d, bs);
    ul.append(li);
  }
}

listen('quick-add', () => { setDefaultTime(); $('title').focus(); $('title').select(); refresh(); });
listen('changed', refresh);
setDefaultTime();
refresh();
setInterval(refresh, 30000);

// ---------- settings ----------
const sSound = $('s-sound'), sVol = $('s-vol'), sRep = $('s-rep'), sFile = $('s-file'), sName = $('s-fname'), sAuto = $('s-auto');

async function loadSettings() {
  const s = await invoke('get_settings');
  sSound.value = s.sound || 'chime';
  sVol.value = s.volume || '0.6';
  sRep.value = s.repeats || '3';
  sName.textContent = s.custom_name || 'none';
  sAuto.checked = s.autostart !== '0';
}
sSound.onchange = () => invoke('set_setting', { key: 'sound', value: sSound.value });
sVol.onchange = () => invoke('set_setting', { key: 'volume', value: sVol.value });
sRep.onchange = () => {
  const n = Math.max(1, Math.min(10, parseInt(sRep.value) || 1));
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
loadSettings();
