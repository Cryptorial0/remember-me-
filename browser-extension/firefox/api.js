import { ext } from './ext.js';

// Talks to the Remember Me? desktop app on this computer (127.0.0.1 only).
const PORTS = Array.from({ length: 10 }, (_, i) => 47821 + i);

async function ping(port) {
  const ctl = new AbortController();
  const timer = setTimeout(() => ctl.abort(), 700);
  try {
    const r = await fetch(`http://127.0.0.1:${port}/ping`, { signal: ctl.signal });
    const j = await r.json();
    return j.app === 'remember-me';
  } catch (e) {
    return false;
  } finally {
    clearTimeout(timer);
  }
}

export async function findPort() {
  const { port } = await ext.storage.local.get('port');
  if (port && (await ping(port))) return port;
  const found = await Promise.all(PORTS.map(async (p) => ((await ping(p)) ? p : null)));
  const p = found.find((x) => x);
  if (p) { await ext.storage.local.set({ port: p }); return p; }
  return null;
}

export async function call(method, path, body) {
  const port = await findPort();
  if (!port) throw new Error('APP_NOT_RUNNING');
  const { token } = await ext.storage.local.get('token');
  if (!token) throw new Error('NO_TOKEN');
  const r = await fetch(`http://127.0.0.1:${port}${path}`, {
    method,
    headers: { 'X-RM-Token': token, 'Content-Type': 'application/json' },
    body: body ? JSON.stringify(body) : undefined,
  });
  if (r.status === 401) throw new Error('BAD_TOKEN');
  if (!r.ok) throw new Error('HTTP_' + r.status);
  return r.json();
}

export const addReminder = (title, due, repeat, link) =>
  call('POST', '/reminders', { title, due, repeat: repeat || 'none', link: link || '' });

/** -> { state: 'ok' | 'no_app' | 'no_token' | 'bad_token', port? } */
export async function checkConnection() {
  const port = await findPort();
  if (!port) return { state: 'no_app' };
  const { token } = await ext.storage.local.get('token');
  if (!token) return { state: 'no_token', port };
  try {
    await call('GET', '/settings');
    return { state: 'ok', port };
  } catch (e) {
    if (e.message === 'BAD_TOKEN') return { state: 'bad_token', port };
    if (e.message === 'APP_NOT_RUNNING') return { state: 'no_app' };
    throw e;
  }
}
