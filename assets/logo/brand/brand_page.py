"""Builds the Phiber brand page from the same geometry as the assets."""
import sys, os, math
sys.argv = ["brand_gen.py", "E"]
sys.path.insert(0, os.path.dirname(__file__))
import brand_gen as g

def svg_of(path):
    return open(path).read()

def inline(svg, cls=""):
    return svg.replace("<svg ", f'<svg class="{cls}" ', 1)

def mark_svg(size, bg=None):
    s = g.wrap(g.warm_l + g.cool_l, bg, size=size)
    return s.replace(' width="%d" height="%d"' % (size, size), "")

MARK = g.wrap(g.warm_l + g.cool_l).replace(' width="512" height="512"', "")
MARK_MONO = svg_of(f"{g.OUT}/mark-mono.svg").replace(' width="512" height="512"', "")
WORD = svg_of(f"{g.OUT}/wordmark.svg")
WORD = WORD[:WORD.index(">") + 1].split(" width=")[0] + ' fill="none" stroke="currentColor">' + WORD[WORD.index(">") + 1:]
LOCK = svg_of(f"{g.OUT}/lockup.svg"); LOCK = LOCK.replace(LOCK[LOCK.index(' width="'):LOCK.index('>')], "")
GLYPHS = {ch: svg_of(f"{g.OUT}/font/{ch}.svg") for ch in "PHIBER"}
for ch in GLYPHS:
    s = GLYPHS[ch]; GLYPHS[ch] = s.replace(s[s.index(' width="'):s.index(' fill=')], "")

# construction drawing: upright mark with dimension callouts
def construction():
    body = g.warm_l + g.cool_l
    grid = "".join(f'<line x1="{i}" y1="0" x2="{i}" y2="24" class="g"/><line x1="0" y1="{i}" x2="24" y2="{i}" class="g"/>' for i in range(0, 25, 2))
    dims = (
        '<line x1="13.2" y1="21.6" x2="13.6" y2="21.6" class="d"/><text x="13.4" y="23.2" class="t">0.4</text>'
        '<line x1="10.8" y1="2.6" x2="13.2" y2="2.6" class="d"/><text x="12" y="1.9" class="t">2.4</text>'
        '<line x1="13.6" y1="2.6" x2="14.9" y2="2.6" class="d"/><text x="14.3" y="1.9" class="t">1.3</text>'
        '<line x1="22.2" y1="4.5" x2="22.2" y2="15.3" class="d"/><text x="22.7" y="10.2" class="t r">10.8</text>'
    )
    return (f'<svg viewBox="-1 0 27.5 24" class="construct" xmlns="http://www.w3.org/2000/svg">{grid}'
            f'<g opacity=".9">{body}</g>{dims}</svg>')

def clearspace():
    body = f'<g transform="rotate(30 12 12)">{g.warm_l}{g.cool_l}</g>'
    return ('<svg viewBox="-6 -6 36 36" class="clear" xmlns="http://www.w3.org/2000/svg">'
            '<rect x="-6" y="-6" width="36" height="36" class="cs"/>'
            '<rect x="0" y="0" width="24" height="24" class="cs2"/>'
            f'{body}<text x="12" y="-2.4" class="t">clear space = ¼ mark</text></svg>')

def tile(bg, fg=None, size=96):
    s = g.wrap(g.warm_l + g.cool_l, bg, size=size)
    return s.replace(f' width="{size}" height="{size}"', "")

SWATCHES = [
    ("Phiber Blue", "#5B9CF0", "cool layer: top inner, bottom C, stem core"),
    ("Phiber Orange", "#F28C28", "warm layer: top C, bottom inner, stem"),
    ("Ink", "#161B22", "dark ground, app chrome"),
    ("Paper", "#F3F5F8", "light ground, wordmark on ink"),
    ("Slate", "#2A3340", "panels, dividers"),
    ("Mist", "#C9D1DC", "secondary text on ink"),
]

LETTER_RULES = {
    "P": "The mark, upright. Bar stem; one half-hex bowl, 8 units tall, detached by one gap.",
    "H": "Two bar stems; the crossbar is E's middle bar and clears each stem by one gap. The left foot and the right head bend inward at P's angle.",
    "I": "One bar stem. The gap and the weight are the only rules it needs.",
    "B": "Two P bowls, each 7.3 units, separated by one gap — B is two Ps.",
    "E": "The B bowl mirrored, its bar stretched to full height; H's crossbar merged into both caps and repeated, detached, in the middle.",
    "R": "B whose lower bowl does not close: its outer side runs straight down to the baseline.",
}

html = f"""<title>Phiber Brand</title>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Chakra+Petch:wght@500;600;700&family=IBM+Plex+Sans:wght@400;500&family=IBM+Plex+Mono:wght@400;500&display=swap">
<style>
:root{{--blue:#5B9CF0;--orange:#F28C28;--ink:#161B22;--paper:#F3F5F8;--slate:#2A3340;--mist:#C9D1DC;
 --bg:#F3F5F8;--fg:#161B22;--muted:#5A6573;--line:#D6DCE4;--card:#FFFFFF;--tile:#E6EAF0;--grid:#D6DCE4;color-scheme:light}}
@media (prefers-color-scheme:dark){{:root:not([data-theme="light"]){{--bg:#161B22;--fg:#F3F5F8;--muted:#9AA6B5;--line:#2A3340;--card:#1C232C;--tile:#222A34;--grid:#2A3340;color-scheme:dark}}}}
:root[data-theme="dark"]{{--bg:#161B22;--fg:#F3F5F8;--muted:#9AA6B5;--line:#2A3340;--card:#1C232C;--tile:#222A34;--grid:#2A3340;color-scheme:dark}}
body{{background:var(--bg);color:var(--fg);font:16px/1.55 "IBM Plex Sans",system-ui,sans-serif;margin:0;padding-block:0 64px;padding-inline:16px}}
.wrap{{max-width:1080px;margin:0 auto}}
h1,h2,h3{{font-family:"Chakra Petch","IBM Plex Sans",sans-serif;text-wrap:balance;margin:0}}
h2{{font-size:1.5rem;font-weight:600;letter-spacing:.01em}}
h3{{font-size:1.05rem;font-weight:600}}
.eyebrow{{font:500 .72rem/1 "IBM Plex Mono",monospace;letter-spacing:.14em;text-transform:uppercase;color:var(--muted)}}
p{{max-width:64ch;margin:0}} .muted{{color:var(--muted)}}
code,.mono{{font-family:"IBM Plex Mono",monospace;font-size:.86em}}
header{{display:grid;grid-template-columns:minmax(0,1fr);gap:24px;padding-block:48px 40px;border-bottom:1px solid var(--line)}}
header .lockup{{width:min(100%,560px);color:var(--fg)}}
header .lockup svg{{width:100%;height:auto;display:block}}
section{{padding-block:40px;border-bottom:1px solid var(--line);display:grid;gap:20px}}
.row{{display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:16px;align-items:start}}
.tiles{{display:grid;grid-template-columns:repeat(auto-fit,minmax(120px,1fr));gap:16px}}
.tiles figure{{margin:0;display:grid;gap:8px}} .tiles svg{{width:100%;height:auto;display:block;border-radius:22.5%}}
figcaption{{font-size:.8rem;color:var(--muted)}}
.construct,.clear{{width:100%;height:auto;display:block;background:var(--tile);border-radius:8px}}
.construct .g{{stroke:var(--grid);stroke-width:.05}} .construct .d{{stroke:var(--fg);stroke-width:.14;stroke-linecap:round}}
.t{{font:600 .95px "IBM Plex Mono",monospace;fill:var(--fg);text-anchor:middle}} .t.r{{text-anchor:start}}
.clear .cs{{fill:var(--tile)}} .clear .cs2{{fill:none;stroke:var(--fg);stroke-width:.12;stroke-dasharray:.5 .5}}
.spec{{display:grid;grid-template-columns:auto 1fr;gap:6px 18px;font-size:.92rem;align-content:start}}
.spec dt{{font-family:"IBM Plex Mono",monospace;font-size:.8rem;color:var(--muted);margin:0;padding-top:2px}} .spec dd{{margin:0}}
.sw{{display:grid;grid-template-columns:repeat(auto-fit,minmax(150px,1fr));gap:14px}}
.sw div{{display:grid;gap:8px}} .sw i{{display:block;height:64px;border-radius:8px;border:1px solid var(--line)}}
.sw b{{font-weight:500}} .sw small{{color:var(--muted);font-size:.8rem;line-height:1.35}}
.glyphs{{display:grid;grid-template-columns:repeat(auto-fit,minmax(150px,1fr));gap:16px}}
.glyphs figure{{margin:0;background:var(--card);border:1px solid var(--line);border-radius:8px;padding:16px;display:grid;gap:10px;color:var(--fg)}}
.glyphs svg{{height:120px;width:auto;max-width:100%;display:block;margin:0 auto}}
.glyphs .ch{{font-family:"Chakra Petch",sans-serif;font-weight:700;font-size:1.1rem}}
.word{{color:var(--fg);background:var(--card);border:1px solid var(--line);border-radius:8px;padding:28px 24px}} .word svg{{width:100%;max-width:640px;height:auto;display:block;margin:0 auto}}
.dont{{display:grid;grid-template-columns:repeat(auto-fit,minmax(150px,1fr));gap:14px}}
.dont figure{{margin:0;display:grid;gap:8px}} .dont .box{{background:var(--tile);border-radius:8px;aspect-ratio:1;display:grid;place-items:center;overflow:hidden;position:relative}}
.dont .box svg{{width:70%;height:auto}} .dont .box::after{{content:"";position:absolute;inset:0;background:linear-gradient(to top right,transparent calc(50% - .5px),#D64545 calc(50% - .5px),#D64545 calc(50% + .5px),transparent calc(50% + .5px))}}
.app{{display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:16px}}
.chrome{{background:var(--ink);color:var(--paper);border-radius:8px;overflow:hidden;border:1px solid var(--slate);font-size:.82rem}}
.bar{{display:flex;align-items:center;gap:10px;padding:6px 10px;background:#1C232C;border-bottom:1px solid var(--slate)}}
.bar svg{{width:20px;height:20px}} .bar span{{opacity:.85}} .bar span.on{{background:var(--slate);padding:2px 8px;border-radius:4px}}
.splash{{padding:18px 16px;display:grid;gap:10px}} .splash .brow{{display:flex;align-items:center;gap:14px}}
.splash .brow svg:first-child{{width:60px;height:60px}} .splash .brow svg:last-child{{height:22px;width:auto;color:var(--paper)}}
.splash .cards{{display:grid;grid-template-columns:1fr 1fr;gap:8px}} .splash .cards div{{background:#1C232C;border-radius:6px;height:54px;border:1px solid var(--slate)}}
footer{{padding-block:28px;font-size:.85rem;color:var(--muted)}}
@media (min-width:720px){{header{{grid-template-columns:1.2fr 1fr;align-items:end}}}}
@media (prefers-reduced-motion:no-preference){{.tiles svg:hover{{transform:translateY(-2px);transition:transform .2s}}}}
</style>
<div class="wrap">
<header>
  <div>
    <div class="eyebrow">Brand guidelines · v1</div>
    <div class="lockup" style="margin-top:14px">{inline(LOCK)}</div>
  </div>
  <p class="muted">One idea drawn three ways: a stem, a half-hexagon bowl, and a constant gap. The mark tilts it 30°, the letters keep it upright, and every piece of the system — the two colours, the spacing, the corner radius — is derived from that geometry rather than added to it.</p>
</header>

<section id="mark">
  <div class="eyebrow">01 · The mark</div>
  <h2>A P, mirrored, in two tones</h2>
  <div class="row">
    <div>{construction()}</div>
    <dl class="spec">
      <dt>canvas</dt><dd>24 × 24 units; the mark is drawn upright and rotated 30° clockwise about the centre</dd>
      <dt>stem</dt><dd>2.4 wide, 4.5 → 19.5; a 1.1 core inset one gap inside it</dd>
      <dt>bowl</dt><dd>half-hexagon C, 10.8 tall, band 1.3, slants at 30°, vertical side 2× a slant, corner radius 1.4</dd>
      <dt>inner</dt><dd>solid half-hex one gap inside the C, cut flat one gap off the stem, corners 0.6</dd>
      <dt>gap</dt><dd><b>0.4</b> everywhere: stem→C, stem→inner, C→inner, core→stem edge</dd>
      <dt>colour</dt><dd>top bowl: orange C, blue inner · bottom bowl: blue C, orange inner · stem orange, core blue</dd>
    </dl>
  </div>
  <p>The gap is the brand. Nothing touches: the bowls float beside the stem, the inner shape floats inside its C, and the core floats inside the stem. If two shapes meet, the mark is wrong.</p>
</section>

<section id="colour">
  <div class="eyebrow">02 · Colourways</div>
  <h2>Two layers, not four colours</h2>
  <div class="tiles">
    <figure>{tile("#161B22")}<figcaption>On Ink — the app icon</figcaption></figure>
    <figure>{tile("#F3F5F8")}<figcaption>On Paper</figcaption></figure>
    <figure>{tile("#5B9CF0")}<figcaption>On Blue — flag use only</figcaption></figure>
    <figure><div style="color:var(--fg)">{tile(None).replace('<svg ', '<svg style="color:currentColor" ', 1) and inline(MARK_MONO)}</div><figcaption>Mono — single ink, favicons ≤16 px</figcaption></figure>
  </div>
  <p>In the app the mark is two SVG layers painted in <code>brand_warm</code> and <code>brand_cool</code>, so a theme can retint it without redrawing. The split follows the colourway (warm = top C + bottom inner + stem; cool = top inner + bottom C + core), not the geometry.</p>
</section>

<section id="palette">
  <div class="eyebrow">03 · Palette</div>
  <h2>Blue and orange, on ink or paper</h2>
  <div class="sw">
    {"".join(f'<div><i style="background:{hex_}"></i><b>{name}</b><span class="mono">{hex_}</span><small>{use}</small></div>' for name, hex_, use in SWATCHES)}
  </div>
  <p>Orange leads: it is the stem and the outer C on top. Blue answers it. Never swap them on one side only — the swapped variant was explored and retired; the official mark is this one.</p>
</section>

<section id="space">
  <div class="eyebrow">04 · Clear space & size</div>
  <h2>Room to float</h2>
  <div class="row">
    <div>{clearspace()}</div>
    <dl class="spec">
      <dt>clear space</dt><dd>a quarter of the mark's width on every side; nothing enters it</dd>
      <dt>minimum</dt><dd>20 px in UI (the header anchor), 16 px favicon in mono, 24 mm in print</dd>
      <dt>rounded square</dt><dd>corner radius 22.5% of the tile (5.4 of 24), mark centred, no padding change</dd>
      <dt>rotation</dt><dd>30° clockwise, always; the upright form is the letter P, not the mark</dd>
    </dl>
  </div>
</section>

<section id="type">
  <div class="eyebrow">05 · Letterforms</div>
  <h2>PHIBER, drawn with the same six rules</h2>
  <div class="word">{WORD}</div>
  <p>One stroke weight (2.0), stems from 4.5 to 19.5, every detached piece one gap (0.4) from its neighbour, every bowl a half-hexagon with the mark's rounded corners. The letters stay upright — the tilt belongs to the mark.</p>
  <div class="glyphs">
    {"".join(f'<figure>{GLYPHS[ch]}<div><span class="ch">{ch}</span><figcaption>{LETTER_RULES[ch]}</figcaption></div></figure>' for ch in "PHIBER")}
  </div>
  <dl class="spec">
    <dt>tracking</dt><dd>2.6 units between letter extents; the lockup sets the wordmark 26 units right of the mark's canvas</dd>
    <dt>lockup</dt><dd>mark first, wordmark cap-height = mark canvas height; use on ink or paper only</dd>
    <dt>files</dt><dd><code>wordmark.svg</code>, <code>font/P.svg … R.svg</code>, <code>lockup-dark.svg</code>, <code>lockup-light.svg</code></dd>
  </dl>
</section>

<section id="app">
  <div class="eyebrow">06 · In the app</div>
  <h2>Where it lives</h2>
  <div class="app">
    <div class="chrome">
      <div class="bar">{MARK}<span class="on">Phiber</span><span>File</span><span>Edit</span><span>Render</span><span>Window</span><span>Help</span></div>
      <div style="padding:10px 12px;color:var(--mist)">Menubar anchor · <code>brand::mark(brand_size)</code> · 20 px, two layers</div>
    </div>
    <div class="chrome">
      <div class="splash">
        <div class="brow">{MARK}{WORD}<span style="margin-left:auto;color:var(--mist)">v0.1.0</span></div>
        <div class="cards"><div></div><div></div><div></div><div></div></div>
      </div>
      <div style="padding:10px 12px;color:var(--mist)">Welcome & About · <code>phiber_splash</code> (3× brand size) + <code>phiber_wordmark</code></div>
    </div>
  </div>
  <dl class="spec">
    <dt>assets</dt><dd><code>assets/icons/phiber.svg</code> (mono) · <code>phiber_warm.svg</code> · <code>phiber_cool.svg</code> · <code>phiber_splash.svg</code> · <code>assets/brand/phiber_wordmark.svg</code></dd>
    <dt>code</dt><dd><code>view/render/gpui/atom/brand.rs</code> resolves the slugs; <code>BView::icon("phiber")</code> is all a view says</dd>
    <dt>theme keys</dt><dd><code>colors.brand_warm</code> (38°, 92%, 50%) · <code>colors.brand_cool</code> (217°, 91%, 60%) · <code>dimensions.brand_size</code> 20</dd>
  </dl>
</section>

<section id="dont">
  <div class="eyebrow">07 · Don't</div>
  <h2>What breaks it</h2>
  <div class="dont">
    <figure><div class="box"><svg viewBox="0 0 24 24"><g transform="rotate(0 12 12)">{g.warm_l}{g.cool_l}</g></svg></div><figcaption>Upright as the mark — that is the letter P</figcaption></figure>
    <figure><div class="box"><svg viewBox="0 0 24 24"><g transform="rotate(30 12 12) scale(1.3 1) translate(-2.8 0)">{g.warm_l}{g.cool_l}</g></svg></div><figcaption>Stretched</figcaption></figure>
    <figure><div class="box"><svg viewBox="0 0 24 24"><g transform="rotate(30 12 12)">{g.warm_l.replace('#F28C28','#5B9CF0')}{g.cool_l.replace('#5B9CF0','#F28C28')}</g></svg></div><figcaption>Colours swapped</figcaption></figure>
    <figure><div class="box"><svg viewBox="0 0 24 24"><g transform="rotate(30 12 12)">{g.warm_l.replace('#F28C28','#8A2BE2')}{g.cool_l.replace('#5B9CF0','#40E0D0')}</g></svg></div><figcaption>Recoloured outside the two brand keys</figcaption></figure>
  </div>
</section>

<footer>Generated from <code>brand_gen.py</code> — the same script that writes every SVG in <code>assets/logo/brand/</code>. Change a number there and the mark, the letters and this page move together.</footer>
</div>
"""
open(f"{g.OUT}/brand.html", "w").write(html)
print("page", len(html))
