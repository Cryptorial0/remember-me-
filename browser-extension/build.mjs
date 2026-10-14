// Builds ready-to-load folders:  chrome/  (Chrome, Brave, Edge)  and  firefox/
// Usage:  node build.mjs     (edit files in shared/ and manifests/, then rebuild)
import { cpSync, rmSync, mkdirSync, copyFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(fileURLToPath(import.meta.url));
for (const target of ['chrome', 'firefox']) {
  const out = join(root, target);
  rmSync(out, { recursive: true, force: true });
  mkdirSync(out, { recursive: true });
  cpSync(join(root, 'shared'), out, { recursive: true });
  copyFileSync(join(root, 'manifests', `${target}.json`), join(out, 'manifest.json'));
  console.log('built', target);
}
