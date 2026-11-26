// The checkout, step by step, checked against the real Phiora with the real shop and catalog.
//
//   node script/checkout.mjs [phiogy's program]
//
// A member puts things in the bag (`--bag SHOP/ID`) and pays (`--pay`); each step is looked at again from a fresh start
// of the program, so what it says is what Phiora has, not what the window remembers:
//   1. a member begins with the wallet the seed gives (2,500 dollars) and owns nothing
//   2. a bag of two themes, a ground and a PFT is paid: Phiora's wallet is down by exactly their prices, and they are owned
//   3. a bag with a look the wallet cannot pay for is refused and nothing is taken
//   4. another member's wallet and things are not touched
//   5. what is owned is not bought again: putting a theme in the bag again and paying takes nothing more
//   6. sixty-four members each pay for a theme: every wallet goes down by the price of a theme, and by that only
// The prices are the shop's own file (`puikit/app/phiogy/public/data/shop/s_phiogy.json`); the words it reads are the program's.
import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
const here = dirname(fileURLToPath(import.meta.url));
const phikit = resolve(here, '../../../../..');
const puikit = resolve(phikit, '../puikit');
const program = resolve(process.argv[2] ?? join(phikit, 'target/main-ui/debug/phiogy.exe'));
const { createPhiora } = await import(pathToFileURL(join(puikit, 'app/phiora-server/src/server.mjs')).href);
const shop = JSON.parse(readFileSync(join(puikit, 'app/phiogy/public/data/shop/s_phiogy.json'), 'utf8'));
const price = (id) => shop.product.find((p) => p.id === id).cost;
const root = mkdtempSync(join(tmpdir(), 'phiogy-checkout-'));
const home = join(root, 'home'); mkdirSync(home);
const items = join(root, 'items'); mkdirSync(items);
writeFileSync(join(items, 'tree.json'), JSON.stringify({ sources: [{ kind: 'person', file: 'core', list: 'users' }] }));
writeFileSync(join(items, 'core.json'), JSON.stringify({ users: [{ id: 'u_noor', name: 'Noor Hale' }] }));
mkdirSync(join(root, 'phiogy/public/data'), { recursive: true });
cpSync(join(puikit, 'app/phiogy/public/data/phiogy.json'), join(root, 'phiogy/public/data/phiogy.json'));
// The catalog and the members are the real ones (Phiegg's data); the records are not needed.
const phiora = createPhiora({ env: { PHI_QUIET: '1', PHI_STORE: 'json' }, dataDir: join(root, 'data'), shopDataDir: join(puikit, 'app/phiegg/src/data'), appsDir: root, itemsDir: items });
await new Promise((ok) => phiora.server.listen(0, ok));
const at = 'localhost:' + phiora.server.address().port;
/** The program as a member, with these flags, and what it said of them. */
const as = (member, ...flags) => new Promise((ok) => execFile(program, ['--phiora', at, '--demo', member, ...flags, '--read'], { encoding: 'utf8', env: { ...process.env, APPDATA: home } }, (error, said, why) => {
  const lines = said.split(/\r?\n/).filter(Boolean);
  const number = (word) => Number((lines.find((l) => l.startsWith(word + ' ')) ?? '').split(' ')[1]);
  ok({ code: error ? error.code ?? 1 : 0, lines, why: why.trim(), wallet: number('wallet'), owned: number('owned') });
}));
const START = 2500;
let failed = false;
try {
  console.log('the program: ' + program + '\nPhiora: http://' + at);
  const first = await as('u_noor');
  assert.deepEqual([first.wallet, first.owned], [START, 0], first.lines.join(' / ') + first.why);
  console.log('1 ok: a member begins with ' + first.wallet + ' dollars and owns ' + first.owned);
  const things = ['theme_sphere', 'ground_ripple', 'pft_bpsc_7001', 'theme_ripple'].filter((id) => shop.product.some((p) => p.id === id));
  const total = things.reduce((n, id) => n + price(id), 0);
  await as('u_noor', ...things.flatMap((id) => ['--bag', 's_phiogy/' + id]), '--pay');
  const paid = await as('u_noor');
  assert.deepEqual([paid.wallet, paid.owned], [START - total, things.length], paid.lines.join(' / '));
  console.log('2 ok: ' + things.length + ' things (' + things.join(', ') + ') cost ' + total + '; Phiora\'s wallet is ' + paid.wallet + ' and they are owned');
  const look = shop.product.find((p) => p.look);
  const refused = await as('u_noor', '--bag', 's_phiogy/' + look.id, '--pay');
  const after = await as('u_noor');
  assert.ok(look.cost > paid.wallet);
  assert.deepEqual([after.wallet, after.owned], [paid.wallet, paid.owned], 'nothing taken: ' + after.lines.join(' / '));
  console.log('3 ok: a look of ' + look.cost + ' with ' + paid.wallet + ' in the wallet is refused, nothing taken');
  const ines = await as('u_ines');
  assert.deepEqual([ines.wallet, ines.owned], [START, 0], 'another member is not touched');
  console.log('4 ok: another member still has ' + ines.wallet + ' and owns ' + ines.owned);
  await as('u_noor', '--bag', 's_phiogy/theme_sphere', '--pay');
  const again = await as('u_noor');
  assert.deepEqual([again.wallet, again.owned], [paid.wallet, paid.owned], 'not bought twice: ' + again.lines.join(' / '));
  console.log('5 ok: a theme already owned is not bought again');
  const sixty = Array.from({ length: 64 }, (_, i) => i);
  const members = (await (await fetch('http://' + at + '/auth/demo-user')).json()).map((u) => u.id);
  assert.equal(members.length, 64);
  const paidOne = [];
  for (const id of members.filter((m) => m !== 'u_noor' && m !== 'u_ines')) {
    await as(id, '--bag', 's_phiogy/theme_sphere', '--pay');
    paidOne.push(await as(id));
  }
  const wrong = paidOne.filter((r) => r.wallet !== START - price('theme_sphere') || r.owned !== 1);
  assert.equal(wrong.length, 0, 'every member paid the price of a theme and no more: ' + wrong.slice(0, 2).map((r) => r.lines.slice(-2).join(' / ')).join(' | '));
  console.log('6 ok: ' + paidOne.length + ' more of the 64 main accounts each paid ' + price('theme_sphere') + ' for a theme: each wallet is ' + (START - price('theme_sphere')) + ' and owns 1');
  console.log('all 6 steps pass');
} catch (error) {
  failed = true; console.error(error);
} finally {
  await new Promise((ok) => phiora.server.close(ok));
  rmSync(root, { recursive: true, force: true });
  process.exit(failed ? 1 : 0);
}
