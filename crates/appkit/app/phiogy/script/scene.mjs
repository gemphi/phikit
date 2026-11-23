// Take a picture of every screen of a call, into doc/scene, each named who-does-what.png.
//
//   node script/scene.mjs            every scene
//   node script/scene.mjs host-receive-all client-hear-ring     only those
//
// Who: the host is the one who begins a call in a room, and whose window serves the line; a client is one who joins
// it, a second window on the host's line (`--at`). The others in a call are callers with no window.
//
// Every scene is staged with the program itself (built with `rtc`), started with what it is to show, and nothing is
// pressed but the bar of a hand's window in one scene: a window that is to be in a call is started with `--join`, and so on. Sound and pictures are the stand-ins
// (`--set device=stand`): a tone and a ramp, each caller's in a tint of its own, so nothing of a room or a screen is
// in a picture. The one scene that asks the system for a camera (`host-lack-camera`) is kept only where the system
// has none: it shows the window saying so, and no picture of anything.
import { execFileSync, spawn } from 'node:child_process';
import { mkdirSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const program = resolve(process.env.PHIOGY || join(here, '../../../../../target/main-ui/debug/phiogy.exe'));
const SNAP = resolve(here, '../../../../../target/phiogy_snap.ps1');
const PRESS = resolve(here, '../../../../../target/phiogy_press.ps1');
const OUT = resolve(here, '../doc/scene');
let PORT = Number(process.env.LINE_PORT || '4311');
const at = () => 'localhost:' + PORT;
const STAND = ['--set', 'device=stand'];
const HAND = ['--width', '390', '--height', '800'];
// The tints of the callers with no window: the blue and the red part of each one's ramp (128 and 128 is grey).
const TINT = [[90, 170], [170, 90], [80, 110], [160, 160], [110, 200]];
const sleep = (time) => new Promise((done) => setTimeout(done, time));

let running = [];
const start = (word) => {
  const one = spawn(program, word, { stdio: ['pipe', 'pipe', 'pipe'] });
  one.said = '';
  one.stdout.on('data', (text) => { one.said += text; });
  one.stderr.on('data', (text) => { one.said += text; });
  running.push(one);
  return one;
};
const window = (room, more = []) => ['--forget', '--phiora', 'none', '--room', room, '--say', ...STAND, ...more];
/** The host's window: it serves the line. */
const host = (room, more) => start([...window(room, more), '--port', String(PORT)]);
/** A client's window: on the host's line. */
const client = (room, more) => start([...window(room, more), '--at', at()]);
/** The line alone, where no host's window serves it. */
const line = () => start(['--line', '--port', String(PORT)]);
/** A caller with no window, its ramp small and in the tint of that number. */
const caller = (room, tint, more = ['--camera']) => start(['--caller', 'phiogy/' + room, '--at', at(), '--for', '120', ...STAND,
  '--set', 'width=320', '--set', 'height=180', '--set', 'frame=8', '--set', 'ramp.blue=' + TINT[tint][0], '--set', 'ramp.red=' + TINT[tint][1], ...more]);
/** Wait until a window in a call says that many others are connected, a little longer for their pictures. */
const connected = async (one, count) => {
  for (let turn = 0; turn < 120; turn += 1) {
    const last = one.said.split('in the call:').pop() ?? '';
    if ((last.match(/: Connected,/g) ?? []).length >= count && one.said.includes('in the call: true')) return sleep(2500);
    await sleep(500);
  }
  throw new Error('never connected to ' + count + ': ' + one.said.slice(-600));
};
// A window that is not up yet is caught as a sliver: such a picture is taken again, a little later.
const snap = (one, name) => {
  for (let turn = 0; turn < 6; turn += 1) {
    const said = execFileSync('powershell', ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', SNAP, '-Id', String(one.pid), '-Out', join(OUT, name + '.png')]).toString().trim();
    const size = / (\d+)x(\d+) /.exec(said);
    if (size && Number(size[1]) >= 300 && Number(size[2]) >= 300) return console.log(said);
    execFileSync('powershell', ['-NoProfile', '-Command', 'Start-Sleep -Milliseconds 1500']);
  }
  throw new Error('no whole picture of ' + name);
};
// A press in a window, at a place measured as a picture of it shows: the one thing a scene presses, where no word of
// the command line leaves the window so (a hand's menu is put away when a room is shown).
const press = (one, across, down) => execFileSync('powershell', ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', PRESS, '-Id', String(one.pid), '-X', String(across), '-Y', String(down)]);
const over = async () => { running.forEach((one) => one.kill()); running = []; await sleep(1500); };

const scene = {
  // A room before a call: the two tools of its title that begin one.
  'host-open-room': async () => { const h = host('lora'); await sleep(5000); snap(h, 'host-open-room'); },
  // A call just begun in a direct room: the host's own tile, the room's person ringing, the call's buttons.
  'host-begin-call': async () => { const h = host('lora', ['--join']); await connected(h, 0); snap(h, 'host-begin-call'); },
  // Begun with the camera.
  'host-begin-video': async () => { const h = host('lora', ['--join', '--camera']); await connected(h, 0); snap(h, 'host-begin-video'); },
  // A client has the room open while somebody calls there: who is calling, and Join.
  'client-hear-ring': async () => { line(); await sleep(1500); caller('lora', 0); await sleep(2500); const c = client('lora'); await sleep(6000); snap(c, 'client-hear-ring'); },
  // Two in a call, both cameras on: each side's window.
  'host-receive-one': async () => {
    const h = host('lobby', ['--join', '--camera']); await connected(h, 0);
    const c = client('lobby', ['--join', '--camera', '--set', 'ramp.blue=' + TINT[0][0], '--set', 'ramp.red=' + TINT[0][1]]);
    await connected(h, 1); await connected(c, 1);
    snap(h, 'host-receive-one'); snap(c, 'client-receive-host');
  },
  // Everybody in a call: the host's window and a client's, one caller muted, one with no camera.
  'host-receive-all': async () => {
    const h = host('lobby', ['--join', '--camera']); await connected(h, 0);
    const c = client('lobby', ['--join', '--camera', '--set', 'ramp.blue=' + TINT[0][0], '--set', 'ramp.red=' + TINT[0][1]]);
    caller('lobby', 1); caller('lobby', 2, ['--camera', '--mute']); caller('lobby', 3, []);
    await connected(h, 4); await connected(c, 4);
    snap(h, 'host-receive-all'); snap(c, 'client-receive-all');
  },
  // The same, the client's window a hand's.
  'client-receive-all-hand': async () => {
    const h = host('lobby', ['--join', '--camera']); await connected(h, 0);
    const c = client('lobby', ['--join', ...HAND]);
    caller('lobby', 1); caller('lobby', 2, ['--camera', '--mute']);
    await connected(h, 3); await connected(c, 3);
    snap(c, 'client-receive-all-hand');
  },
  // The host muted: its own tile says so, and the button offers the way back.
  'host-mute-self': async () => {
    const h = host('lobby', ['--join', '--camera', '--mute']); await connected(h, 0); caller('lobby', 1);
    await connected(h, 1); snap(h, 'host-mute-self');
  },
  // The system asked for its camera where it has none: the window says so and the call goes on. The microphone is
  // open for these few seconds; nothing is played, and nothing of it is kept.
  'host-lack-camera': async () => {
    const h = start(['--forget', '--phiora', 'none', '--room', 'lora', '--say', '--port', String(PORT), '--join', '--camera', '--set', 'play.loud=0']);
    await connected(h, 0);
    if (/not given: .*camera/.test(h.said.split('in the call:').pop())) snap(h, 'host-lack-camera');
    else console.log('host-lack-camera: this system has a camera, so no picture is kept');
  },
  // A voice room seen from outside: who its file names, who is in it on its line, and the way in.
  'client-see-voice': async () => { line(); await sleep(1500); caller('lounge', 0, []); caller('lounge', 1, ['--mute']); await sleep(2500); const c = client('lounge'); await sleep(6000); snap(c, 'client-see-voice'); },
  // In a voice room: the person among those in it, the microphone, the sound of the others.
  'host-enter-voice': async () => { const h = host('lounge', ['--join']); await connected(h, 0); caller('lounge', 0, []); caller('lounge', 1, ['--mute']); await connected(h, 2); snap(h, 'host-enter-voice'); },
  // Muted there, and the sound of the others off.
  'host-deafen-voice': async () => { const h = host('lounge', ['--join', '--mute', '--deaf']); await connected(h, 0); caller('lounge', 0, []); await connected(h, 1); snap(h, 'host-deafen-voice'); },
  // Still in the voice room while another room is looked at: the room at the foot of the menu, with its microphone,
  // the others' sound and the way out. And the same muted, the sound off, in a hand's window with the menu drawn out.
  'host-keep-voice': async () => { const h = host('lounge', ['--join', '--then', 'lobby']); await connected(h, 0); caller('lounge', 0, []); await connected(h, 1); snap(h, 'host-keep-voice'); },
  'host-keep-voice-hand': async () => {
    const h = host('lounge', ['--join', '--mute', '--deaf', '--then', 'lobby', ...HAND]); await connected(h, 0); caller('lounge', 0, []); await connected(h, 1);
    // The window is up once a picture of it was taken; then the bar's first row is pressed, which draws its menu out.
    snap(h, 'host-keep-voice-hand'); press(h, 36, 763); await sleep(2000); snap(h, 'host-keep-voice-hand');
  },
};

mkdirSync(OUT, { recursive: true });
const asked = process.argv.slice(2);
let failed = 0;
for (const [name, stage] of Object.entries(scene)) {
  if (asked.length && !asked.includes(name)) continue;
  // A window now and then fails to open (the system refuses its renderer a path): the scene is staged again.
  let done = false;
  for (let turn = 0; turn < 3 && !done; turn += 1) {
    try { await stage(); done = true; } catch (error) { console.error(name + ': ' + error.message.split(/\r?\n/)[0].slice(0, 200)); }
    await over();
    // The next line is on a port of its own: the last one's is not free at once.
    PORT += 1;
  }
  failed += done ? 0 : 1;
}
process.exit(failed ? 1 : 0);
