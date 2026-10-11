const { invoke } = window.__TAURI__.core;
const q = new URLSearchParams(location.search);
const id = Number(q.get('id'));
document.getElementById('t').textContent = q.get('title') || '';

(async () => {
  let st = { sound: 'chime', volume: '0.6', repeats: '3' };
  let custom = null;
  try {
    st = { ...st, ...(await invoke('get_settings')) };
    if (st.sound === 'custom') custom = await invoke('get_custom_sound');
  } catch (e) {}
  const n = Math.max(1, Math.min(10, parseInt(st.repeats) || 1));
  let i = 0;
  const go = () => {
    Sound.play(st.sound, parseFloat(st.volume), custom);
    if (++i < n) setTimeout(go, 10000);
  };
  go();
})();

const close = () => invoke('close_popup');
document.getElementById('done').onclick = async () => { await invoke('toggle_done', { id }); close(); };
document.getElementById('s10').onclick = async () => { await invoke('snooze', { id, minutes: 10 }); close(); };
document.getElementById('x').onclick = close;
