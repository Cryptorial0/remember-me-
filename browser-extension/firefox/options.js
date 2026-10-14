import { checkConnection } from './api.js';
import { ext } from './ext.js';

const $ = (id) => document.getElementById(id);

async function test() {
  const r = $('result');
  r.className = '';
  r.textContent = 'Testing...';
  let st;
  try { st = await checkConnection(); } catch (e) { st = { state: 'no_app' }; }
  const msgs = {
    ok: 'Connected to the app (port ' + st.port + '). You are all set.',
    no_app: 'Cannot find the app. Make sure Remember Me? is running.',
    no_token: 'Paste the token first.',
    bad_token: 'The app did not accept that token. Copy it again from the app.',
  };
  r.textContent = msgs[st.state] || 'Unknown problem.';
  r.className = st.state === 'ok' ? 'good' : 'bad';
}

$('save').onclick = async () => {
  await ext.storage.local.set({ token: $('token').value.trim() });
  test();
};

(async () => {
  const { token } = await ext.storage.local.get('token');
  if (token) { $('token').value = token; test(); }
})();
