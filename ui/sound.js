const Sound = (() => {
  let ctx, master = null, audio = null;
  const ac = () => {
    if (!ctx) ctx = new (window.AudioContext || window.webkitAudioContext)();
    ctx.resume();
    return ctx;
  };
  function tone(c, f, start, dur, type, vol, dest) {
    const o = c.createOscillator(), g = c.createGain();
    o.type = type; o.frequency.value = f;
    g.gain.setValueAtTime(0.0001, start);
    g.gain.linearRampToValueAtTime(vol, start + 0.015);
    g.gain.exponentialRampToValueAtTime(0.0001, start + dur);
    o.connect(g); g.connect(dest);
    o.start(start); o.stop(start + dur + 0.05);
  }
  const builtin = {
    chime:   (c, t, v, d) => [[880, 0], [1318.5, .18], [1760, .36]].forEach(([f, o]) => tone(c, f, t + o, .6, 'sine', .3 * v, d)),
    bell:    (c, t, v, d) => [[523, 1], [1046, .6], [1568, .4], [2093, .25]].forEach(([f, a]) => tone(c, f, t, 1.6, 'sine', .25 * a * v, d)),
    beep:    (c, t, v, d) => [0, .35].forEach((o) => tone(c, 1000, t + o, .18, 'square', .15 * v, d)),
    digital: (c, t, v, d) => [800, 1000, 1200, 1000, 1200, 1500].forEach((f, i) => tone(c, f, t + i * .09, .08, 'square', .12 * v, d)),
    soft:    (c, t, v, d) => { tone(c, 520, t, 1.0, 'sine', .3 * v, d); tone(c, 780, t + .25, 1.0, 'sine', .2 * v, d); },
  };
  function stop() {
    try {
      if (audio) { audio.pause(); audio.currentTime = 0; audio = null; }
      if (master) { master.disconnect(); master = null; }
    } catch (e) {}
  }
  function play(name, volume, customUrl) {
    try {
      stop();
      if (name === 'none') return;
      const vol = Math.min(1, Math.max(0, Number(volume) || 0.6));
      if (name === 'custom' && customUrl) {
        audio = new Audio(customUrl);
        audio.volume = vol;
        audio.play().catch(() => {});
        return;
      }
      const c = ac();
      master = c.createGain();
      master.connect(c.destination);
      (builtin[name] || builtin.chime)(c, c.currentTime + 0.02, vol, master);
    } catch (e) {}
  }
  return { play, stop };
})();
