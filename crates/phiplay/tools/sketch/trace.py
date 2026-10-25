import numpy as np, json, sys
from PIL import Image
from scipy import ndimage
K = 2576/2000
CROPS = {
  # name: (file, x0, y0, x1, y1) in the 2000-wide display coordinates
  'tall_side':   ('views.jpg', 90, 145, 540, 965),
  'tall_inner':  ('views.jpg', 465, 145, 890, 965),
  'tall_front':  ('views.jpg', 955, 145, 1200, 975),
  'tall_back':   ('views.jpg', 1285, 145, 1525, 975),
  'tall_sole':   ('views.jpg', 1665, 520, 1855, 975),
  'tall_top':    ('views.jpg', 1620, 145, 1895, 480),
  'mid_side':    ('cutoff.jpg', 465, 380, 870, 925),
  'ankle_front': ('cutoff.jpg', 960, 555, 1155, 980),
  'slip_side':   ('cutoff.jpg', 1220, 755, 1630, 952),
  'thigh_side':  ('extended.jpg', 465, 255, 880, 915),
  'knee_side':   ('extended.jpg', 90, 150, 540, 965),
  'flat_side':   ('extended.jpg', 1005, 610, 1495, 958),
  'ankle_side':  ('orange.jpg', 990, 440, 1215, 920),
}
def shape(name):
    f, x0, y0, x1, y1 = CROPS[name]
    im = np.asarray(Image.open(f).convert('RGB')).astype(int)
    c = im[int(y0*K):int(y1*K), int(x0*K):int(x1*K)]
    line = (c[:,:,0] > 110) & (c[:,:,0] - c[:,:,2] > -60)
    grow = 6 if name in ('ankle_front','slip_side','flat_side','ankle_side') else 3
    line = ndimage.binary_dilation(line, iterations=grow)
    free = ~line
    lab, n = ndimage.label(free)
    border = set(lab[0,:]) | set(lab[-1,:]) | set(lab[:,0]) | set(lab[:,-1])
    outside = np.isin(lab, list(border - {0}))
    s = ~outside
    # keep the largest component
    lab2, n2 = ndimage.label(s)
    if n2 > 1:
        sizes = ndimage.sum(s, lab2, range(1, n2+1))
        s = lab2 == (1 + int(np.argmax(sizes)))
    s = ndimage.binary_fill_holes(s)
    s = ndimage.binary_erosion(s, iterations=grow - 1)
    return s
if __name__ == '__main__':
    tiles = []
    for name in CROPS:
        s = shape(name)
        np.save(name + '.npy', s)
        im = Image.fromarray((s*255).astype(np.uint8)).convert('RGB')
        im.thumbnail((300, 300)); tiles.append(im)
    W = Image.new('RGB', (300*7, 300*2), (40,40,40))
    for i, t in enumerate(tiles): W.paste(t, ((i%7)*300, (i//7)*300))
    W.save('masks.png')
