const { invoke } = window.__TAURI__.core;
const q = new URLSearchParams(location.search);
const id = Number(q.get('id'));
document.getElementById('t').textContent = q.get('title') || '';

let timer = null;
let stopped = false;

(async () => {
  let st = { sound: 'chime', volume: '0.6', repeats: '0' };
  let custom = null;
  try {
    st = { ...st, ...(await invoke('get_settings')) };
    if (st.sound === 'custom') custom = await invoke('get_custom_sound');
  } catch (e) {}
  const r = parseInt(st.repeats);
  const n = isNaN(r) ? 0 : Math.max(0, Math.min(10, r));
  const total = n === 0 ? 30 : n; // 0 = until dismissed (stops after ~3 minutes)
  let i = 0;
  const go = () => {
    if (stopped) return;
    Sound.play(st.sound, parseFloat(st.volume), custom);
    if (++i < total) timer = setTimeout(go, 6000);
  };
  go();
})();

const close = () => invoke('close_popup');
document.getElementById('done').onclick = async () => { await invoke('toggle_done', { id }); close(); };
document.getElementById('s10').onclick = async () => { await invoke('snooze', { id, minutes: 10 }); close(); };
document.getElementById('x').onclick = close;
document.getElementById('m').onclick = () => { stopped = true; clearTimeout(timer); Sound.stop(); };
