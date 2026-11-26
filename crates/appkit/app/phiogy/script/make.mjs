// Clubs and PFTs published from a sheet, checked against the real Phiora.
//
//   node script/make.mjs [phiogy's program]
//
// The Node Phiora (`puikit/app/phiora-server`) is started here on a port of its own, with a store of its own in a
// folder that is removed after, and the program (`phiogy --make SHEET`) is run against it, with its own folder too:
//   1. as nobody (a guest) it publishes nothing, and says to sign in
//   2. as a member, `--dry` says what it would make and makes nothing
//   3. as that member it makes the club and its two PFTs, each PFT's clubId the id Phiora gave the club
//   4. the same sheet again makes nothing: the three are the same
//   5. one cell changed in the sheet changes that one record, and only it
//   7. the window's drawers (--fill, --publish) make a club and a PFT in it, say the same again changes nothing, and refuse a club that is not the person's
//   8. a host made in the window (`--host`) is published on Phiora; a second member (`--adopt`) joins it and has it with its rooms; the same name makes one host
//   9. what a member writes in a room is a record on Phiora, and another member of the room reads it, once
//   10. a friend request is a record on Phiora that the other member is asked by, accepts, and either removes
//   11. a like given to a message is counted by another member of the room and taken back; a follow is a record
//   6. a row that is wrong is named and the rest are still published; the program ends saying it failed
// What Phiegg's web reads is Phiora's `/tree`: the club and the PFTs are there, as the web's own boot overlays them.
// The words it looks for are the program's own, `data/make.tsv`.
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
const table = (name) => Object.fromEntries(readFileSync(join(here, '../data', name), 'utf8').split(/\r?\n/).filter((l) => l && !l.startsWith('#')).map((l) => l.split('\t')));
const word = table('make.tsv');
const words = table('text.tsv');
const text = (name) => words[name];
const sheet = readFileSync(join(here, '../doc/club-sheet.tsv'), 'utf8');
const root = mkdtempSync(join(tmpdir(), 'phiogy-make-'));
const home = join(root, 'home'); mkdirSync(home);
const shop = join(root, 'shop'); mkdirSync(shop);
writeFileSync(join(shop, 'core.json'), JSON.stringify({ users: [{ id: 'u_noor', name: 'Noor Hale', handle: 'noor.hale' }, { id: 'u_ines', name: 'Ines Valcourt', handle: 'ines.v' }] }));
const items = join(root, 'items'); mkdirSync(items);
writeFileSync(join(items, 'tree.json'), JSON.stringify({ sources: [{ kind: 'person', file: 'core', list: 'users' }, { kind: 'club', file: 'core', list: 'clubs' }, { kind: 'profilePft', file: 'assets', list: 'profiles' }, { kind: 'host', file: 'hosts', list: 'hosts' }, { kind: 'message', file: 'messages', list: 'messages' }, { kind: 'bond', file: 'bonds', list: 'bonds' }, { kind: 'mark', file: 'marks', list: 'marks' }, { kind: 'follow', file: 'follows', list: 'follows' }] }));
writeFileSync(join(items, 'marks.json'), JSON.stringify({ marks: [] }));
writeFileSync(join(items, 'follows.json'), JSON.stringify({ follows: [] }));
writeFileSync(join(items, 'bonds.json'), JSON.stringify({ bonds: [] }));
writeFileSync(join(items, 'messages.json'), JSON.stringify({ messages: [] }));
writeFileSync(join(items, 'hosts.json'), JSON.stringify({ hosts: [] }));
writeFileSync(join(items, 'core.json'), JSON.stringify({ users: [{ id: 'u_noor', name: 'Noor Hale' }, { id: 'u_ines', name: 'Ines Valcourt' }], clubs: [] }));
writeFileSync(join(items, 'assets.json'), JSON.stringify({ profiles: [] }));
// The window takes Phiora for its own only where Phiora gives it its document, so Phiora is given the web Phiogy's.
mkdirSync(join(root, 'phiogy/public/data'), { recursive: true });
cpSync(join(puikit, 'app/phiogy/public/data/phiogy.json'), join(root, 'phiogy/public/data/phiogy.json'));
const phiora = createPhiora({ env: { PHI_QUIET: '1', PHI_STORE: 'json' }, dataDir: join(root, 'data'), shopDataDir: shop, appsDir: root, itemsDir: items });
await new Promise((ok) => phiora.server.listen(0, ok));
const at = 'localhost:' + phiora.server.address().port;
/** The program, run on a sheet: what it said, what it said of failing, and how it ended. */
const make = (file, ...more) => new Promise((ok) => execFile(program, ['--phiora', at, '--make', file, ...more], { encoding: 'utf8', env: { ...process.env, APPDATA: home } }, (error, said, why) => ok({ code: error ? error.code ?? 1 : 0, lines: said.split(/\r?\n/).filter(Boolean), why: why.trim() })));
const records = async () => (await (await fetch('http://' + at + '/tree?kind=club,profilePft')).json()).sources.flatMap((s) => (s.rows ?? []).map((r) => ({ kind: s.kind, ...r })));
const fill = (name, values) => Object.entries(values).reduce((line, [k, v]) => line.replaceAll('{' + k + '}', v), word[name]);
const write = (name, text) => { const file = join(root, name); writeFileSync(file, text); return file; };
try {
  console.log('the program: ' + program + '\nPhiora: http://' + at);
  const file = write('club-sheet.tsv', sheet);
  const guest = await make(file, '--signout');
  assert.equal(guest.code, 1);
  assert.equal(guest.why, word['say.nobody']);
  assert.deepEqual(await records(), []);
  console.log('1 ok: a guest publishes nothing: "' + guest.why + '"');
  const dry = await make(file, '--demo', 'u_noor', '--dry');
  assert.equal(dry.code, 0, dry.why);
  assert.deepEqual(dry.lines.slice(0, 3), [fill('say.would', { kind: 'club', ref: 'sheet_club' }), fill('say.would', { kind: 'profilePft', ref: 'sheet_pft_1' }), fill('say.would', { kind: 'profilePft', ref: 'sheet_pft_2' })]);
  assert.deepEqual(await records(), []);
  console.log('2 ok: --dry says "' + dry.lines[0] + '" and makes nothing');
  const real = await make(file, '--demo', 'u_noor');
  assert.equal(real.code, 0, real.why + real.lines.join('\n'));
  const on = await records();
  const club = on.find((r) => r.kind === 'club' && r.ref === 'sheet_club');
  const pups = on.filter((r) => r.kind === 'profilePft');
  assert.ok(club, 'the club is on Phiora');
  assert.equal(club.name, 'Sheet Club');
  assert.equal(club.supply, 2);
  assert.deepEqual(club.tags, ['pfp', 'sheet']);
  assert.equal(pups.length, 2);
  assert.ok(pups.every((p) => p.clubId === club.id && p.ownerId === club.ownerId), 'each PFT names the club by the id Phiora gave it');
  assert.deepEqual(pups.find((p) => p.ref === 'sheet_pft_1').traits, { background: '#140d38', outfit: '#565656', collection: 'Sheet Club' });
  assert.equal(pups.find((p) => p.ref === 'sheet_pft_2').n, 2);
  assert.equal(real.lines.at(-1), fill('say.done', { made: 3, changed: 0, same: 0, refused: 0 }));
  console.log('3 ok: ' + real.lines.at(-1) + '; the PFTs name the club by its id ' + club.id);
  const again = await make(file, '--demo', 'u_noor');
  assert.equal(again.lines.at(-1), fill('say.done', { made: 0, changed: 0, same: 3, refused: 0 }));
  assert.equal((await records()).length, 3);
  console.log('4 ok: the same sheet again: ' + again.lines.at(-1));
  const edited = write('edited.tsv', sheet.replace('A club made from a sheet.', 'A club, said again.'));
  const changed = await make(edited, '--demo', 'u_noor');
  assert.deepEqual(changed.lines, [fill('say.changed', { kind: 'club', ref: 'sheet_club', fields: 'tagline' }), fill('say.same', { kind: 'profilePft', ref: 'sheet_pft_1' }), fill('say.same', { kind: 'profilePft', ref: 'sheet_pft_2' }), fill('say.done', { made: 0, changed: 1, same: 2, refused: 0 })]);
  assert.equal((await records()).find((r) => r.ref === 'sheet_club').tagline, 'A club, said again.');
  console.log('5 ok: one cell changed, one record changed: "' + changed.lines[0] + '"');
  const broken = write('broken.tsv', sheet.split('\n').filter((l) => !l.includes('sheet_pft_2')).join('\n') + 'profilePft\tsheet_pft_3\tSheet Pup #3\t\t\t\t\t\t\t\t@nobody\t3\n');
  const bad = await make(broken, '--demo', 'u_noor');
  assert.equal(bad.code, 1);
  assert.ok(bad.lines.some((l) => l === fill('say.refused', { kind: 'profilePft', ref: 'sheet_pft_3', why: fill('why.link', { name: 'nobody' }) })), bad.lines.join('\n'));
  assert.ok(!(await records()).some((r) => r.ref === 'sheet_pft_3'));
  console.log('6 ok: a PFT of a club nobody has is refused, the rest stand: "' + bad.lines.find((l) => l.startsWith('refused')) + '"');
  // The window's drawers (`--fill` and `--publish` say what a drawer holds and press its Publish).
  const drawer = (kind, ...fills) => new Promise((ok) => execFile(program, ['--phiora', at, '--demo', 'u_noor', '--publish', kind, ...fills.flatMap((f) => ['--fill', f])], { encoding: 'utf8', env: { ...process.env, APPDATA: home } }, (error, said, why) => ok({ code: error ? error.code ?? 1 : 0, said: said.trim(), why: why.trim() })));
  const made = await drawer('newclub', 'newclub/name=Drawer Club', 'newclub/ticker=DRW', 'newclub/supply=10');
  assert.equal(made.said, fill('say.made', { kind: 'club', ref: 'club_drawer_club', id: (await records()).find((r) => r.ref === 'club_drawer_club')?.id }), made.said + made.why);
  const dc = (await records()).find((r) => r.ref === 'club_drawer_club');
  assert.deepEqual([dc.name, dc.ticker, dc.supply], ['Drawer Club', 'DRW', 10]);
  const pft = await drawer('newpft', 'newpft/name=Drawer Pup #7', 'newpft/number=7', 'newpft/club=' + dc.id);
  const dp = (await records()).find((r) => r.ref === 'pft_drawer_club_0007');
  assert.ok(dp, pft.said + pft.why);
  assert.deepEqual([dp.clubId, dp.n, dp.set, dp.tokenId, dp.name], [dc.id, 7, 'Drawer Club', 'drawer-club-0007', 'Drawer Pup #7']);
  const again7 = await drawer('newclub', 'newclub/name=Drawer Club', 'newclub/ticker=DRW', 'newclub/supply=10');
  assert.equal(again7.said, fill('say.same', { kind: 'club', ref: 'club_drawer_club' }));
  const stranger = await drawer('newpft', 'newpft/name=Lost', 'newpft/number=1', 'newpft/club=not-a-club');
  assert.equal(stranger.said, text('publish.club'));
  console.log('7 ok: the drawers publish what the command line does: "' + made.said + '", "' + pft.said + '", then "' + again7.said + '"');  // A host made in the window is published, and another member finds it and joins it (`--host`, `--adopt`, `--read`).
  const run = (member, ...flags) => new Promise((ok) => execFile(program, ['--phiora', at, '--demo', member, ...flags, ...(flags.includes('--read') ? [] : ['--read'])], { encoding: 'utf8', env: { ...process.env, APPDATA: home } }, (error, said, why) => ok({ code: error ? error.code ?? 1 : 0, lines: said.split(/\r?\n/).filter(Boolean), why: why.trim() })));
  const hostMade = await run('u_noor', '--host', 'Moon Room');
  assert.ok(hostMade.lines[0].startsWith('made host m'), hostMade.lines.join('\n') + hostMade.why);
  const hosts = async () => (await (await fetch('http://' + at + '/tree?kind=host')).json()).sources.flatMap((s) => (s.rows ?? []).filter((r) => r.ref));
  const moon = (await hosts()).find((h) => h.name === 'Moon Room');
  assert.ok(moon && Array.isArray(moon.rooms) && moon.rooms.length >= 1 && moon.ownerId, 'the host is on Phiora with its rooms: ' + JSON.stringify(moon) + ' | said: ' + hostMade.lines.join(' / ') + hostMade.why);
  await run('u_noor', '--host', 'Moon Room');
  assert.equal((await hosts()).filter((h) => h.name === 'Moon Room').length, 1, 'the same name makes no second host');
  const before8 = await run('u_ines', '--read');
  assert.ok(!before8.lines.some((l) => l.includes('Moon Room')), 'Ines is not in it yet: ' + before8.lines.filter((l) => l.includes('Moon')).join(' / '));
  const joined = await run('u_ines', '--adopt', 'Moon Room');
  assert.equal(joined.lines[0], `joined Moon Room, with ${moon.rooms.length} rooms`, joined.lines.join('\n') + joined.why);
  const after8 = await run('u_ines', '--read');
  assert.ok(after8.lines.some((l) => l.includes('Moon Room')), after8.lines.join('\n'));
  const nobody = await run('u_ines', '--adopt', 'No Such Host');
  assert.equal(nobody.lines[0], text('host.none'));
  console.log('8 ok: a host made by one member is on Phiora with its ' + moon.rooms.length + ' rooms, a second member joins it and has it in their rail, a name made twice is one host');
  // What a member writes in a room reaches another member of it (`--write`, `--room`, `--read` says what is in the room).
  const heard = (lines) => lines.filter((l) => l.includes('hello from noor'));
  const wrote = await run('u_noor', '--room', 'lobby', '--write', 'hello from noor');
  const records9 = async () => (await (await fetch('http://' + at + '/tree?kind=message')).json()).sources.flatMap((s) => s.rows ?? []);
  const sent9 = (await records9()).find((m) => m.text === 'hello from noor');
  assert.ok(sent9 && sent9.room && sent9.authorName === 'Noor Hale' && sent9.ownerId, 'the message is a record on Phiora: ' + wrote.lines.join(' / ') + wrote.why);
  const inboxOfInes = await run('u_ines', '--room', 'lobby', '--read');
  assert.equal(heard(inboxOfInes.lines).length, 1, 'Ines reads what Noor wrote: ' + inboxOfInes.lines.slice(-6).join(' / '));
  assert.ok(heard(inboxOfInes.lines)[0].includes('Noor Hale'), heard(inboxOfInes.lines)[0]);
  const noorAgain = await run('u_noor', '--room', 'lobby', '--read');
  assert.equal(heard(noorAgain.lines).length, 1, 'Noor sees it once, not twice: ' + heard(noorAgain.lines).join(' / '));
  console.log('9 ok: a message written by one member is a record on Phiora and is read by another in the same room: "' + heard(inboxOfInes.lines)[0].trim() + '"');
  // A friend request reaches the other member, who answers it (`--bond ID=state`, `--stand ID`).
  const stand = async (member, other, ...more) => (await run(member, ...more, '--stand', other)).lines.find((l) => l.startsWith(other + ' ')).split(' ')[1];
  assert.equal(await stand('u_ines', 'u_noor'), 'none', 'strangers');
  assert.equal(await stand('u_noor', 'u_ines', '--bond', 'u_ines=sent'), 'sent', 'Noor asked');
  const bonds = async () => (await (await fetch('http://' + at + '/tree?kind=bond')).json()).sources.flatMap((s) => s.rows ?? []);
  const req = (await bonds()).find((b) => b.state === 'sent');
  assert.ok(req && req.userId === req.to && req.from !== req.to, 'the request is a record on Phiora naming the one asked: ' + JSON.stringify(req));
  assert.equal(await stand('u_ines', 'u_noor'), 'asked', 'Ines was asked, on another run, by what Phiora has');
  assert.equal(await stand('u_ines', 'u_noor', '--bond', 'u_noor=with'), 'with', 'Ines accepts');
  assert.equal(await stand('u_noor', 'u_ines'), 'with', 'Noor is told: they are friends');
  assert.equal((await bonds()).filter((b) => b.state === 'with').length, 1, 'one record says so');
  assert.equal(await stand('u_noor', 'u_ines', '--bond', 'u_ines=none'), 'none', 'Noor removes the friend');
  assert.equal(await stand('u_ines', 'u_noor'), 'none', 'Ines is told: not friends');
  console.log('10 ok: a friend request is a record on Phiora, the other member is asked, accepts, both are friends, and one removes the other');
  // A like given to a message is counted by the others in the room (`--give ID`), and a follow is a record (`Follow`, by `--follow`).
  const liked = (lines) => lines.filter((l) => /heart \d+: hello from noor/.test(l));
  const message = (await records9()).find((m) => m.text === 'hello from noor');
  const noorSees0 = await run('u_noor', '--room', 'lobby', '--read');
  assert.equal(liked(noorSees0.lines).length, 0, 'no like yet');
  await run('u_ines', '--room', 'lobby', '--give', message.ref);
  const marks = (await (await fetch('http://' + at + '/tree?kind=mark')).json()).sources.flatMap((s) => s.rows ?? []);
  assert.equal(marks.length, 1, 'the like is one record on Phiora');
  assert.deepEqual([marks[0].messageId, marks[0].icon, marks[0].state, marks[0].room], [message.ref, 'heart', 'on', message.room]);
  const noorSees = await run('u_noor', '--room', 'lobby', '--read');
  assert.ok(liked(noorSees.lines).some((l) => l.includes('heart 1:')), 'Noor sees one like from Ines: ' + noorSees.lines.slice(-4).join(' / '));
  await run('u_ines', '--room', 'lobby', '--give', message.ref);
  assert.equal((await (await fetch('http://' + at + '/tree?kind=mark')).json()).sources.flatMap((s) => s.rows ?? []).filter((r) => r.state === 'on').length, 0, 'taken back, it is off, still one record');
  assert.equal(liked((await run('u_noor', '--room', 'lobby', '--read')).lines).length, 0, 'and Noor no longer sees it');
  await run('u_noor', '--follow', 'u_ines');
  const follows = (await (await fetch('http://' + at + '/tree?kind=follow')).json()).sources.flatMap((s) => s.rows ?? []);
  assert.equal(follows.length, 1);
  assert.deepEqual([follows[0].state, follows[0].to === follows[0].from ? 'self' : 'other'], ['on', 'other']);
  console.log('11 ok: a like given by one member is counted by another in the room, and taken back; a follow is a record on Phiora');
  console.log('all 11 steps pass');
} catch (error) {
  console.error(error);
  process.exitCode = 1;
} finally {
  await new Promise((ok) => phiora.server.close(ok));
  rmSync(root, { recursive: true, force: true });
  process.exit(process.exitCode ?? 0);
}
