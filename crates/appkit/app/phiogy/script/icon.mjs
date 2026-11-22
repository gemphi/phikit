// Write lucide's drawings as Phiogy's own icons, from the copy of lucide the web kit is built on.
//
//   node script/icon.mjs reply heart pin
//
// The web Phiogy draws with lucide (`puikit/node_modules/lucide-react`). The icons every appkit app shares
// (`crates/appkit/asset/icon`) are lucide's too, and do not have every one the web's rooms use. This writes the ones
// named, each as `asset/icon/NAME.svg`, in the form of the shared ones: one drawing on a square of 24, in the ink it
// is tinted with. Nothing is fetched: a name lucide's copy on this disk does not have is said, and nothing is written
// for it. The licence is lucide's, the shared icons' own (`crates/appkit/asset/icon/LICENSE`).
import { existsSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const lucide = resolve(here, '../../../../../../puikit/node_modules/lucide-react/dist/esm/icons');
const HEAD = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">';

let missing = 0;
for (const name of process.argv.slice(2)) {
  const file = join(lucide, name + '.mjs');
  if (!existsSync(file)) { console.error('lucide has no ' + name + ' here'); missing += 1; continue; }
  const { __iconNode: node } = await import(pathToFileURL(file).href);
  // A name that is only another icon's second name has no drawing of its own.
  if (!node) { console.error(name + ' is a second name of another icon in lucide: ask for that one'); missing += 1; continue; }
  const drawn = node.map(([tag, given]) => '<' + tag + Object.entries(given).filter(([key]) => key !== 'key').map(([key, value]) => ` ${key}="${value}"`).join('') + '/>').join('');
  writeFileSync(join(here, '../asset/icon', name + '.svg'), HEAD + drawn + '</svg>\n');
  console.log('wrote asset/icon/' + name + '.svg');
}
process.exit(missing ? 1 : 0);
