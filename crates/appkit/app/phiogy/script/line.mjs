// Run the Node line's own tests against Phiogy's line, unchanged.
//
//   node script/line.mjs [phiogy's program] [the test file]
//
// TEST names another of the Node server's test files, and ONLY the cases of it to run (a pattern of their names):
// the line answers an app's document, `/me` and a person's documents as well, and `test/server.test.mjs` has cases
// for those.
//
// `puikit/app/phiora-server/test/call.test.mjs` starts the Node Phiora and
// asks it what a page asks. Here the same file is run with one line of it
// changed: where it imports the Node server, it imports a stand-in that
// starts Phiogy's line instead (`phiogy --line --port 0`) and hands the
// test the port it listens on. Every case and every assertion is the
// test's own, so what passes here is what the web pages rely on.
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const phikit = resolve(here, '../../../../..');
const program = resolve(process.argv[2] ?? join(phikit, 'target/main-ui/debug/phiogy.exe'));
const test = resolve(process.argv[3] ?? process.env.TEST ?? join(phikit, '../puikit/app/phiora-server/test/call.test.mjs'));
/** The line of the test that brings the Node server in. */
const IMPORT = /^import \{ createPhiora \} from '[^']+';$/m;

// The stand-in: what the test calls on the Node server, done by Phiogy's
// line. The line says where it listens on its first line.
const standIn = `
import { spawn } from 'node:child_process';
export function createPhiora({ env = {}, appsDir } = {}) {
  let child; let port;
  const server = {
    listen(_, ok) {
      child = spawn(${JSON.stringify(program)}, ['--line', '--port', '0'], { env: { ...process.env, ...env, ...(appsDir ? { PHI_APPS: appsDir } : {}) }, stdio: ['ignore', 'pipe', 'inherit'] });
      let said = '';
      child.stdout.on('data', (piece) => {
        said += piece; const found = said.match(/:(\\d+)\\s/);
        if (found && !port) { port = Number(found[1]); ok(); }
      });
      child.on('exit', (code) => { if (!port) throw new Error('the line did not open: ' + code); });
    },
    address: () => ({ port }),
    close() { child?.kill(); },
  };
  return { server, calls: { close() {} } };
}
`;

const source = readFileSync(test, 'utf8');
if (!IMPORT.test(source)) { console.error('The test no longer imports createPhiora as it did: ' + test); process.exit(2); }
const folder = mkdtempSync(join(tmpdir(), 'phiogy-line-'));
const shim = join(folder, 'phiora.mjs'); const copy = join(folder, 'call.test.mjs');
writeFileSync(shim, standIn);
writeFileSync(copy, source.replace(IMPORT, `import { createPhiora } from ${JSON.stringify(pathToFileURL(shim).href)};`));
console.log('the line: ' + program + '\nthe test: ' + test);
const ran = spawnSync(process.execPath, ['--test', ...(process.env.ONLY ? ['--test-name-pattern', process.env.ONLY] : []), copy], { stdio: 'inherit' });
rmSync(folder, { recursive: true, force: true });
process.exit(ran.status ?? 1);
