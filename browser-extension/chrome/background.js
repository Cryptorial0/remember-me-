import { ext } from './ext.js';

// Right-click menu: "Remind me about this ..." opens a small window with the form pre-filled.
async function setupMenus() {
  await ext.contextMenus.removeAll();
  ext.contextMenus.create({ id: 'rm-selection', title: 'Remind me about "%s"', contexts: ['selection'] });
  ext.contextMenus.create({ id: 'rm-link', title: 'Remind me about this link', contexts: ['link'] });
  ext.contextMenus.create({ id: 'rm-page', title: 'Remind me about this page', contexts: ['page'] });
}

ext.runtime.onInstalled.addListener(async (details) => {
  await setupMenus();
  if (details.reason === 'install') ext.runtime.openOptionsPage();
});
ext.runtime.onStartup.addListener(setupMenus);

ext.contextMenus.onClicked.addListener((info, tab) => {
  let title = '';
  let link = '';
  if (info.menuItemId === 'rm-selection') {
    title = (info.selectionText || '').trim().slice(0, 200);
    link = info.pageUrl || (tab && tab.url) || '';
  } else if (info.menuItemId === 'rm-link') {
    link = info.linkUrl || '';
    title = (info.selectionText || link).slice(0, 200);
  } else {
    title = (tab && tab.title) || '';
    link = info.pageUrl || (tab && tab.url) || '';
  }
  const q = new URLSearchParams({ title, link, ctx: '1' });
  ext.windows.create({
    url: ext.runtime.getURL('popup.html?' + q.toString()),
    type: 'popup',
    width: 380,
    height: 520,
  });
});
