// A call between Phiogy in Rust, as a caller, and one Chrome page of the web Phiogy.
//
//   node script/peer.mjs [phiogy's program]
//
// Needs the web Phiogy served with its API at this line: the launch entry `phiogy-line` (vite on 3121,
// VITE_PHI_API=http://localhost:4311). This starts `phiogy --caller phiogy/lora --port 4311`, which serves the line
// and joins the room's call on it with no window, then one headless Chrome with Chrome's own stand-in camera and
// microphone, opens the room, presses Join with a real click, and prints what each side hears and sees of the other.
// VOICE names a sound file Chrome plays as its microphone; without one Chrome's stand-in microphone may be silent.
// The page's side is measured in the page; Rust's side is what the caller prints each second. The steps are those of
// the two-Chrome check of the web's own calls (phiogy-call-check.mjs), with one Chrome replaced by Rust.
import { execFileSync, spawn } from 'node:child_process';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const program = resolve(process.argv[2] ?? join(here, '../../../../../target/main-ui/debug/phiogy.exe'));
const PAGE = process.env.CALL_URL || 'http://localhost:3121/phiogy/#/lora';
const CHROME = process.env.CHROME || 'C:/Program Files/Google/Chrome/Application/chrome.exe';
const PORT = process.env.LINE_PORT || '4311';
const sleep = (ms) => new Promise((ok) => setTimeout(ok, ms));
const say = (k, v) => console.log(k.padEnd(52), typeof v === 'object' ? JSON.stringify(v) : v);

// Rust's side: the caller, and everything it says.
const said = [];
// WINDOW=1 has the real window be the caller (`--join --say`); without it the caller has no window and can be told
// on its standard input to mute, unmute and leave, which a window needs a hand for.
const WINDOW = !!process.env.WINDOW;
// DEVICE=system has Rust use the system's own microphone, camera and sound out; without it Rust sends its stand-ins
// (a tone and a grey ramp) and plays nothing. LOUD is the share of full what arrives is played at (system only).
// With the system's devices no picture is kept: SHOT is not taken, since it would show the room.
const DEVICE = process.env.DEVICE || 'stand';
const SET = ['--set', 'device=' + DEVICE, '--set', 'play.loud=' + (process.env.LOUD || '0.1'), '--camera'];
// WINSHOT names a file a picture of the Rust window is kept in while both cameras are on (WINDOW, stand-ins only);
// MORE is more flags for the window, with spaces between (`--share`, `--width 420 --height 760`).
const MORE = (process.env.MORE || '').split(' ').filter(Boolean);
const snap = (file) => execFileSync('powershell', ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', resolve(here, '../../../../../target/phiogy_snap.ps1'), '-Id', String(rust.pid), '-Out', resolve(file)]).toString().trim();
const rust = spawn(program, WINDOW ? ['--forget', '--room', 'lora', '--join', '--say', '--port', PORT, ...SET, ...MORE] : ['--caller', 'phiogy/lora', '--port', PORT, '--for', '150', ...SET], { stdio: ['pipe', 'pipe', 'pipe'] });
const tell = (word) => rust.stdin.write(word + '\n');
rust.stdout.on('data', (d) => String(d).split(/\r?\n/).filter(Boolean).forEach((l) => said.push(l)));
rust.stderr.on('data', (d) => String(d).split(/\r?\n/).filter(Boolean).forEach((l) => said.push('! ' + l)));
/** The last thing Rust said of the other caller: its stage, how loud, how many pictures. */
const far = () => {
  const l = [...said].reverse().find((x) => /: (New|Connecting|Connected|Failed),/.test(x)); if (!l) return null;
  const m = l.match(/: (\w+), muted (\w+), loud ([\d.]+), pictures (\d+) \((\d+) x (\d+)\)/);
  return m && { stage: m[1], muted: m[2] === 'true', loud: Number(m[3]), pictures: Number(m[4]), size: m[5] + ' x ' + m[6] };
};

// The page's side: one Chrome of its own, driven over the DevTools protocol.
const profile = mkdtempSync(join(tmpdir(), 'phiogy-peer-'));
const chrome = spawn(CHROME, ['--headless=new', '--remote-debugging-port=9335', '--user-data-dir=' + profile, '--no-first-run', '--window-size=1280,860',
  '--use-fake-device-for-media-stream', '--use-fake-ui-for-media-stream', '--auto-select-desktop-capture-source=Entire screen', ...(process.env.VOICE ? ['--use-file-for-fake-audio-capture=' + process.env.VOICE] : []), 'about:blank'], { stdio: 'ignore' });
const end = (code) => { chrome.kill(); rust.kill(); setTimeout(() => { try { rmSync(profile, { recursive: true, force: true }); } catch { /* still held */ } process.exit(code); }, 500); };
let ws;
for (let i = 0; i < 40 && !ws; i++) { await sleep(250); try { const v = await (await fetch('http://127.0.0.1:9335/json/version')).json(); ws = new WebSocket(v.webSocketDebuggerUrl); } catch { /* not up yet */ } }
await new Promise((ok, no) => { ws.onopen = ok; ws.onerror = no; });
let seq = 0; const waiting = new Map(); const log = [];
ws.onmessage = (e) => {
  const m = JSON.parse(e.data);
  if (m.id && waiting.has(m.id)) { const [ok, no] = waiting.get(m.id); waiting.delete(m.id); m.error ? no(new Error(m.error.message)) : ok(m.result); }
  else if (m.method === 'Runtime.exceptionThrown') log.push('EXCEPTION ' + (m.params.exceptionDetails.exception?.description ?? m.params.exceptionDetails.text));
};
const cdp = (method, params = {}, sessionId) => new Promise((ok, no) => { const id = ++seq; waiting.set(id, [ok, no]); ws.send(JSON.stringify({ id, method, params, sessionId })); });
const { targetId } = await cdp('Target.createTarget', { url: 'about:blank' });
const { sessionId } = await cdp('Target.attachToTarget', { targetId, flatten: true });
await cdp('Runtime.enable', {}, sessionId); await cdp('Page.enable', {}, sessionId);
await cdp('Page.navigate', { url: PAGE }, sessionId);
const ev = async (expr) => { const r = await cdp('Runtime.evaluate', { expression: expr, awaitPromise: true, returnByValue: true }, sessionId); if (r.exceptionDetails) throw new Error(r.exceptionDetails.text); return r.result.value; };
const until = async (what, expr, ms = 20000) => { const t = Date.now(); for (;;) { const v = typeof expr === 'function' ? expr() : await ev(expr); if (v) return v; if (Date.now() - t > ms) throw new Error('never ' + what); await sleep(200); } };
// An element by its aria-label, or a button by its words; in the call's own group first.
const FIND = `(l) => { const call = document.querySelector('[role="group"][aria-label^="Calling"]'); const inCall = call && [...call.querySelectorAll('button')].find((e) => e.textContent.trim() === l); return inCall ?? document.querySelector('[aria-label="' + l + '"]') ?? [...document.querySelectorAll('button')].find((e) => e.textContent.trim() === l); }`;
const click = async (label) => {
  const at = await until('found "' + label + '"', `(() => { const e = (${FIND})(${JSON.stringify(label)}); if (!e) return null; e.scrollIntoView({ block: 'center' }); const r = e.getBoundingClientRect(); return { x: r.x + r.width / 2, y: r.y + r.height / 2 }; })()`);
  for (const type of ['mouseMoved', 'mousePressed', 'mouseReleased']) await cdp('Input.dispatchMouseEvent', { type, x: at.x, y: at.y, button: type === 'mouseMoved' ? 'none' : 'left', clickCount: 1 }, sessionId);
};
const inCall = (label) => `[...document.querySelectorAll('[role="group"][aria-label^="Calling"] button')].some((e) => e.textContent.trim() === ${JSON.stringify(label)})`;
const press = async (label, becomes) => { for (let i = 0; i < 4; i++) { await click(label); try { await until('"' + label + '" took', becomes, 2500); return; } catch (e) { if (i === 3) throw e; } } };
const moving = (label) => ev(`(async () => { const v = (${FIND})(${JSON.stringify(label)}); if (!v || !v.videoWidth) return false; const t = v.currentTime; await new Promise((ok) => setTimeout(ok, 800)); return v.currentTime > t ? v.videoWidth + ' x ' + v.videoHeight : false; })()`);
const loud = (label) => ev(`(async () => { const v = (${FIND})(${JSON.stringify(label)}); const s = v && v.srcObject; if (!s || !s.getAudioTracks().length) return -1;
  const ctx = (window.__ctx ??= new AudioContext()); await ctx.resume(); const ear = ctx.createAnalyser(); ear.fftSize = 2048; const src = ctx.createMediaStreamSource(s); src.connect(ear);
  const data = new Float32Array(ear.fftSize); let top = 0; const end = Date.now() + 3000;
  while (Date.now() < end) { ear.getFloatTimeDomainData(data); for (const x of data) top = Math.max(top, Math.abs(x)); await new Promise((ok) => setTimeout(ok, 50)); }
  src.disconnect(); return Math.round(top * 1000) / 1000; })()`);

let failed = false;
const step = async (k, f) => { try { say(k, await f()); } catch (e) { failed = true; say(k, 'FAILED: ' + e.message); } };
try {
  await until('the room shown', `!!document.querySelector('[aria-label^="Search "]')`, 40000);
  await step('Rust: it says it joined', () => until('joined', () => said.find((l) => l.includes('is joining'))));
  await step('page: "Guest is calling" with Join', () => until('incoming', `document.body.innerText.includes('Guest is calling')`));
  await click('Join');
  await step('page: in the call', () => until('in call', `document.body.innerText.includes('In the call')`));
  await step('Rust: connected to the page', () => until('connected', () => far()?.stage === 'Connected' && far()));
  await step('page sees Rust\'s picture moving', () => until('far picture', `!!document.querySelector('[aria-label="The camera of Guest"]')`).then(() => sleep(1500)).then(() => moving('The camera of Guest')));
  await step('page: the size of Rust\'s picture', () => ev(`(() => { const v = (${FIND})('The camera of Guest'); return v ? v.videoWidth + ' x ' + v.videoHeight : 'none'; })()`));
  await step('page hears Rust (loudest of 3 s, 1 is full)', () => loud('The sound of Guest'));
  await step('Rust hears the page (loudest, 1 is full)', () => until('heard', () => (far()?.loud ?? 0) > 0 && far().loud, 15000));
  await press('Camera on', inCall('Camera off'));
  await step('Rust sees the page\'s camera', () => until('pictures', () => (far()?.pictures ?? 0) > 5 && far(), 20000));
  const camera = far()?.size;
  if (WINDOW && process.env.WINSHOT && DEVICE === 'stand') await step('a picture of the Rust window', () => sleep(1500).then(() => snap(process.env.WINSHOT)));
  const { data } = await cdp('Page.captureScreenshot', { format: 'png' }, sessionId);
  if (process.env.SHOT && DEVICE === 'stand') writeFileSync(process.env.SHOT, Buffer.from(data, 'base64'));
  const muted = `[...document.querySelectorAll('[role="group"][aria-label^="Calling"] span')].some((e) => e.textContent.trim() === 'Muted')`;
  if (!WINDOW) {
    tell('mute');
    await step('Rust muted: the page\'s tile reads Muted', () => until('muted word', muted));
    await step('Rust muted: the page hears (0 is silence)', () => sleep(1000).then(() => loud('The sound of Guest')));
    tell('unmute');
    await step('Rust unmuted: the tile no longer reads Muted', () => until('unmuted', `!(${muted})`));
    await step('Rust unmuted: the page hears again', () => sleep(1500).then(() => loud('The sound of Guest')));
    const theirs = `!!document.querySelector('[aria-label="The camera of Guest"]')`;
    tell('still');
    await step('Rust camera off: the page shows its face, no camera', () => until('camera gone', `!(${theirs})`, 20000));
    tell('camera');
    await step('Rust camera on again: the page sees it moving', () => until('camera back', theirs, 20000).then(() => sleep(1500)).then(() => moving('The camera of Guest')));
    const scene = `(() => { const v = (${FIND})('The screen Guest shares'); return v && v.videoWidth ? v.videoWidth + ' x ' + v.videoHeight : ''; })()`;
    tell('share');
    await step('Rust shares a screen: the page sees it', () => until('their screen', scene, 20000));
    await step('Rust\'s screen at the page, a few seconds on', () => sleep(4000).then(() => ev(scene)));
    tell('unshare');
    await step('Rust stops sharing: the page has no screen of it', () => until('screen gone', `!document.querySelector('[aria-label="The screen Guest shares"]')`, 20000));
    await step('Rust says of itself', () => [...said].reverse().find((l) => l.includes('this caller:')));
  }
  await press('Share a screen', inCall('Stop sharing'));
  await step('page shares a screen: Rust gets a second picture', () => until('screen', () => far() && far().size !== camera && far(), 20000));
  await click('Mute');
  await step('page muted: Rust is told, and hears nothing', () => until('muted', () => far()?.muted && far().loud === 0 && far(), 15000));
  await click('End the call');
  await step('page left: Rust is alone', () => until('alone', () => [...said].reverse().find((l) => l.includes('others:'))?.includes('others: 0')));
  if (!WINDOW) {
    await step('page: Rust is still calling, with Join', () => until('incoming again', `document.body.innerText.includes('Guest is calling')`));
    const before = said.length;
    await step('page: joins again, and is in the call', () => press('Join', `document.body.innerText.includes('In the call')`).then(() => true));
    await step('Rust: connected to the page again', () => until('connected again', () => said.slice(before).find((l) => /: Connected,/.test(l)), 20000));
    tell('leave');
    await step('Rust left: the page\'s tile for it is gone', () => until('tile gone', `!document.body.innerText.includes('In the call') && !document.querySelector('[aria-label="The camera of Guest"]')`));
  }
  say('what Rust said last', said.slice(-3));
  say('console of the page', log.slice(0, 8));
} catch (e) { failed = true; console.log('STOPPED:', e.message); say('what Rust said last', said.slice(-6)); }
if (process.env.ALL) console.log(said.join('\n'));
end(failed ? 1 : 0);
