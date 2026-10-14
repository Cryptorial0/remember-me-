import { checkConnection, addReminder } from './api.js';
import { ext } from './ext.js';

const $ = (id) => document.getElementById(id);
const pad = (n) => String(n).padStart(2, '0');
const toInput = (d) =>
  `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
const isHttp = (s) => /^https?:\/\//i.test(s || '');

function atHour(h, addDays) {
  const d = new Date();
  d.setHours(h, 0, 0, 0);
  if (addDays) d.setDate(d.getDate() + addDays);
  return d;
}
function tonight() { const d = atHour(20, 0); if (d <= new Date()) d.setDate(d.getDate() + 1); return d; }
function tomorrow() { return atHour(9, 1); }
function whenFor(key) {
  if (key === 'tonight') return tonight();
  if (key === 'tomorrow') return tomorrow();
  if (key === '1h') return new Date(Date.now() + 3600000);
  return new Date(Date.now() + Number(key) * 60000);
}

const params = new URLSearchParams(location.search);
const inWindow = params.get('ctx') === '1';
let page = { title: '', url: '' };

function say(text, bad) {
  const m = $('msg');
  m.textContent = text;
  m.className = bad ? 'bad' : 'good';
}

function errorText(e) {
  const m = e && e.message;
  if (m === 'APP_NOT_RUNNING') return 'Remember Me? is not running. Start the app and try again.';
  if (m === 'NO_TOKEN' || m === 'BAD_TOKEN') return 'Token problem. Open settings and paste the token again.';
  return 'Could not save (' + m + ').';
}

async function save(title, due, repeat, link) {
  try {
    await addReminder(title.trim().slice(0, 500), due, repeat, isHttp(link) ? link : '');
    say('Saved: ' + new Date(due * 1000).toLocaleString());
    setTimeout(() => window.close(), 1100);
  } catch (e) {
    say(errorText(e), true);
  }
}

function showStatus(state) {
  $('status').hidden = false;
  const text = $('status-text'), btn = $('status-btn');
  if (state === 'no_app') {
    text.textContent = 'Remember Me? is not running. Start the app, then try again.';
    btn.textContent = 'Try again';
    btn.onclick = () => location.reload();
  } else {
    text.textContent = state === 'bad_token'
      ? 'The app did not accept the token. Copy it again from the app.'
      : 'One-time setup: paste the secret token from the app.';
    btn.textContent = 'Open setup';
    btn.onclick = () => ext.runtime.openOptionsPage();
  }
}

async function init() {
  let st;
  try { st = await checkConnection(); } catch (e) { st = { state: 'no_app' }; }
  if (st.state !== 'ok') return showStatus(st.state);
  $('main').hidden = false;

  if (inWindow) {
    page = { title: params.get('title') || '', url: params.get('link') || '' };
  } else {
    const [t] = await ext.tabs.query({ active: true, currentWindow: true });
    if (t) page = { title: t.title || '', url: t.url || '' };
  }
  $('title').value = page.title;
  $('link').value = isHttp(page.url) ? page.url : '';
  $('when').value = toInput(new Date(Date.now() + 10 * 60000));
  $('title').focus();
  $('title').select();
}

document.querySelectorAll('.later button').forEach((b) =>
  b.addEventListener('click', () => {
    if (!isHttp(page.url)) return say('This page has no web link to save. Use the form below.', true);
    const title = 'Read: ' + (page.title || page.url);
    save(title, Math.floor(whenFor(b.dataset.later).getTime() / 1000), 'none', page.url);
  })
);

document.querySelectorAll('#f .chips button').forEach((b) =>
  b.addEventListener('click', () => { $('when').value = toInput(whenFor(b.dataset.m)); })
);

$('f').addEventListener('submit', (e) => {
  e.preventDefault();
  const due = Math.floor(new Date($('when').value).getTime() / 1000);
  save($('title').value, due, $('repeat').value, $('link').value.trim());
});

init();
