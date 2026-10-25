"""Writes each sketch view the footwear was read from as a small PNG for
Phiber's sketch check, and the silhouette's box in it."""
import numpy as np
from PIL import Image
from trace import CROPS, K
OUT = '/home/claude/ws/crates/phiber/assets/footwear/sketch'
USED = ['tall_side', 'tall_inner', 'tall_front', 'tall_back', 'tall_sole', 'thigh_side', 'mid_side', 'ankle_front', 'slip_side', 'flat_side']
rows = []
for name in USED:
    f, x0, y0, x1, y1 = CROPS[name]
    im = Image.open(f).convert('RGB').crop((int(x0*K), int(y0*K), int(x1*K), int(y1*K)))
    s = np.load(name + '.npy')
    scale = min(1.0, 560 / im.height)
    im = im.resize((round(im.width * scale), round(im.height * scale)), Image.LANCZOS)
    ys, xs = np.nonzero(s)
    box = [round(v * scale) for v in (xs.min(), ys.min(), xs.max(), ys.max())]
    im.save(f'{OUT}/{name}.png', optimize=True)
    rows.append((name, box))
    print(name, im.size, box)
open('boxes.txt', 'w').write('\n'.join(f'{n} {b}' for n, b in rows))
