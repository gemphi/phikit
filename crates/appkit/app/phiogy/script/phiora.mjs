// What Phiogy's window reads, checked against the real Phiora.
//
//   node script/phiora.mjs [phiogy's program]
//
// The Node Phiora (`puikit/app/phiora-server`) is started here on a port of its own, with a store of its own in a
// folder that is removed after, and seeded from a copy of the web Phiogy's own file, as it seeds itself. Then the
// program is asked what it read (`phiogy --read`), four times:
//   1. of Phiora: it says so, and reads the hosts and rooms the files give, since Phiora was seeded from them
//   2. after a room is called otherwise in Phiora (an edit to its menu record, as an editor makes one): the
//      window's rail has the new name, which no file has
//   3. of nobody (`--phiora none`): the files, with the old name
//   4. of Phiora after it is closed: the files, and it says Phiora did not answer
// And, before Phiora is closed, twice more with something written in a room (`--write`):
//   5. what was written is in the room at once, and on Phiora in the person's own document, as the web keeps it
//   6. with the folder's copy lost it is read back from Phiora by the same session
//   7. the program is one of Phiora's demo members (`--demo`), with that member's own documents, her
//      sessions and everything Phiora keeps about her taken out (`--export`), and a guest again after (`--signout`)
//   8. a sign-in with a handle nobody has and nonsense for a password (`--signin`) is refused, in Phiora's words
// No account is made and no real password is used: the members are the seed's, who have none.
// `SHOT=path` also takes a picture of the window's account while that member is signed in.
// The words it looks for are the program's own, read from `data/text.tsv`, `data/say.tsv` and `data/phiora.tsv`.
import assert from 'node:assert/strict';
import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { execFile } from 'node:child_process';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const phikit = resolve(here, '../../../../..');
const puikit = resolve(phikit, '../puikit');
const program = resolve(process.argv[2] ?? join(phikit, 'target/main-ui/debug/phiogy.exe'));
const { createPhiora } = await import(pathToFileURL(join(puikit, 'app/phiora-server/src/server.mjs')).href);

/** A table of the program's, by name. */
const table = (name) => Object.fromEntries(readFileSync(join(here, '../data', name), 'utf8').split(/\r?\n/).filter((l) => l && !l.startsWith('#')).map((l) => l.split('\t')));
/** The program's own words, and its word for asking nobody. */
const text = { ...table('text.tsv'), ...table('say.tsv') };
const NOBODY = table('phiora.tsv').none;
const from = (name, at) => text['read.from'].replace('{at}', text[name].replace('{at}', at));
const room = (id, label) => text['read.room'].replace('{id}', id).replace('{label}', label);

/** What the program says it read, as lines. It is waited for without stopping this process, which Phiora answers in. */
const read = (...word) => new Promise((ok, fail) => execFile(program, ['--read', ...word], { encoding: 'utf8', env: { ...process.env, APPDATA: home, PHIOGY_SECRET: NONSENSE } }, (error, said, why) => (error ? fail(new Error(why || String(error))) : ok(said.split(/\r?\n/).filter(Boolean)))));

const root = mkdtempSync(join(tmpdir(), 'phiogy-phiora-'));
// What is sent as a password in the one sign-in tried here: nonsense, for a handle nobody has. No account is made.
const NONSENSE = 'not-a-password-' + Date.now();
// The person's own folder, for the program: what it keeps is kept here, not in the real one.
const home = join(root, 'home'); mkdirSync(home);
const KEY = 'phiogy';
const file = join(puikit, 'app', KEY, 'public/data', KEY + '.json');
mkdirSync(join(root, KEY, 'public/data'), { recursive: true });
cpSync(file, join(root, KEY, 'public/data', KEY + '.json'));
// A person who may edit, and the least a Phiora needs to start.
const shop = join(root, 'shop'); mkdirSync(shop);
writeFileSync(join(shop, 'core.json'), JSON.stringify({ users: [{ id: 'u_noor', name: 'Noor Hale', handle: 'noor.hale' }] }));
const items = join(root, 'items'); mkdirSync(items);
writeFileSync(join(items, 'tree.json'), JSON.stringify({ sources: [{ kind: 'person', file: 'core', list: 'users' }] }));
writeFileSync(join(items, 'core.json'), JSON.stringify({ users: [{ id: 'u_noor', name: 'Noor Hale' }] }));

const phiora = createPhiora({ env: { PHI_QUIET: '1', PHI_STORE: 'json', PHI_EDITORS: '*' }, dataDir: join(root, 'data'), shopDataDir: shop, appsDir: root, itemsDir: items });
await new Promise((ok) => phiora.server.listen(0, ok));
const at = 'localhost:' + phiora.server.address().port;
let cookie = '';
const call = async (method, path, body) => {
  const r = await fetch('http://' + at + path, { method, headers: { origin: 'http://localhost:3118', ...(cookie ? { cookie } : {}), ...(body !== undefined ? { 'content-type': 'application/json' } : {}) }, body: body === undefined ? undefined : JSON.stringify(body) });
  const set = r.headers.get('set-cookie'); if (set) cookie = set.split(';')[0];
  const said = await r.text(); return { status: r.status, body: said ? JSON.parse(said) : undefined };
};
let closed = false;
try {
  console.log('the program: ' + program + '\nPhiora: http://' + at);
  // Phiora composes the app's document from its records: the same document the file is.
  assert.deepEqual((await call('GET', '/app/' + KEY)).body, JSON.parse(readFileSync(file, 'utf8')));

  const files = await read('--forget', '--phiora', NOBODY);
  assert.equal(files[0], from('fact.file', ''));
  const first = await read('--forget', '--phiora', at);
  assert.equal(first[0], from('fact.phiora', at));
  assert.deepEqual(first.slice(1), files.slice(1));
  console.log('1 ok: ' + first[0] + ' (' + (first.length - 1) + ' lines of hosts and rooms, the files\' own)');

  // A room of the rail is called otherwise in Phiora: an edit to its menu record.
  const menus = (await call('GET', '/tree?kind=menu')).body.sources.find((s) => s.kind === 'menu').rows;
  const lobby = menus.find((m) => m.key === 'lobby');
  assert.ok(lobby, 'the rail has a lobby');
  await call('GET', '/me'); await call('POST', '/auth/demo', { userId: 'u_noor' });
  assert.equal((await call('PATCH', '/tree/item/menu/' + lobby.id, { set: { label: 'front room' } })).status, 200);
  const edited = await read('--forget', '--phiora', at);
  assert.ok(edited.includes(room('lobby', 'front room')), edited.join('\n'));
  assert.ok(!edited.includes(room('lobby', lobby.label)));
  console.log('2 ok: after the edit in Phiora the window reads "' + room('lobby', 'front room').trim() + '"');

  const still = await read('--forget', '--phiora', NOBODY);
  assert.deepEqual(still, files);
  assert.ok(still.includes(room('lobby', lobby.label)));
  console.log('3 ok: asked of nobody it reads the files: "' + room('lobby', lobby.label).trim() + '"');

  // What is written in a room is said in it at once and kept on Phiora, under the session Phiora gave the program.
  const WORDS = 'Kept on Phiora ' + Date.now();
  const said = (lines) => lines.filter((l) => l.includes(WORDS)).length;
  const wrote = await read('--phiora', at, '--room', 'lobby', '--write', WORDS);
  assert.equal(said(wrote), 1, wrote.slice(-4).join(' / '));
  const keep = table('keep.tsv'); const folder = join(home, table('app.tsv').folder);
  const session = readFileSync(join(folder, keep.session), 'utf8').trim();
  assert.ok(session.startsWith('phi_sid='), 'the session is kept');
  const onPhiora = await (await fetch('http://' + at + '/state/' + keep['doc.sent'], { headers: { cookie: session } })).json();
  assert.equal(onPhiora[KEY + '/lobby'].at(-1).text, WORDS);
  assert.equal(onPhiora[KEY + '/lobby'].at(-1).by, 'you');
  console.log('5 ok: what was written is on Phiora, in the document "' + keep['doc.sent'] + '" of the session Phiora gave the program');
  // The folder's own copy lost, the next time it is read from Phiora by the same session; nobody else has it.
  writeFileSync(join(folder, keep.file), '{}');
  assert.equal(said(await read('--phiora', at, '--room', 'lobby')), 1);
  assert.ok(existsSync(join(folder, keep.file)));
  assert.equal(said(await read('--phiora', NOBODY, '--room', 'lobby', '--forget')), 0);
  console.log('6 ok: read again from Phiora after the copy in the folder was lost; asked of nobody, with nothing kept, it is not there');

  // Being one of Phiora's members to try things: the session Phiora gave becomes theirs, and stays so.
  const you = (lines) => lines.find((l) => l.startsWith(text['read.me'].replace('{name}', ''))) ?? '';
  assert.ok(you(await read('--phiora', at)).includes(text['account.guest']));
  assert.ok(you(await read('--phiora', at, '--demo', 'u_noor')).includes('Noor Hale'));
  const still7 = await read('--phiora', at, '--room', 'lobby');
  assert.ok(you(still7).includes('Noor Hale'));
  // Their documents are their own: what the guest wrote is not theirs.
  assert.equal(said(still7), 0);
  const me = await (await fetch('http://' + at + '/me', { headers: { cookie: readFileSync(join(folder, keep.session), 'utf8').trim() } })).json();
  assert.equal(me.handle, 'noor.hale');
  // Signed in, the sessions signed in as her are counted, and everything Phiora keeps about her is taken out.
  const out = join(root, 'export.json');
  const signed = await read('--phiora', at, '--export', out);
  // Two: this check's own, which signed in to make the edit of step 2, and the program's.
  assert.ok(signed.includes(text['read.session'].replace('{count}', '2')), signed.slice(0, 3).join(' / '));
  const kept = JSON.parse(readFileSync(out, 'utf8'));
  assert.equal(kept.user.handle, 'noor.hale');
  assert.ok(kept.exportedAt && !('passwordHash' in kept.user));
  // The window itself, with Phiora answering and somebody signed in: a picture of its account, when one is asked for.
  if (process.env.SHOT) {
    const shot = resolve(phikit, 'target/phiogy_shot.ps1');
    const asked = `& '${shot}' -Out '${resolve(process.env.SHOT)}' -Argument '--phiora','${at}','--open','account','--wheel','-4000' -Wait 3`;
    await new Promise((ok) => execFile('powershell', ['-NoProfile', '-Command', asked], { env: { ...process.env, APPDATA: home } }, (error, said) => { console.log(String(said).trim()); ok(); }));
  }
  // Nobody again: a guest, with another session.
  assert.ok(you(await read('--phiora', at, '--signout')).includes(text['account.guest']));
  assert.notEqual(readFileSync(join(folder, keep.session), 'utf8').trim(), session);
  console.log('7 ok: the program was Noor Hale to Phiora, with her own documents, her two sessions counted, her data taken out, and a guest again after');
  // A handle nobody has, with nonsense for a password: Phiora refuses, and the program says what Phiora said.
  const refused = await read('--phiora', at, '--signin', 'nobody.here');
  assert.ok(refused.some((l) => l === text['read.refused'].replace('{why}', 'Handle or password is wrong.')), refused.slice(0, 3).join(' / '));
  assert.ok(you(refused).includes(text['account.guest']));
  console.log('8 ok: a sign-in Phiora refuses is said in its words, and the program stays a guest');

  phiora.server.closeAllConnections?.(); await new Promise((ok) => phiora.server.close(ok)); closed = true;
  const after = await read('--forget', '--phiora', at);
  assert.equal(after[0], from('fact.unanswered', at));
  assert.deepEqual(after.slice(1), files.slice(1));
  console.log('4 ok: ' + after[0]);
} finally {
  if (!closed) { phiora.server.closeAllConnections?.(); phiora.server.close(); }
  rmSync(root, { recursive: true, force: true });
}
process.exit(0);
