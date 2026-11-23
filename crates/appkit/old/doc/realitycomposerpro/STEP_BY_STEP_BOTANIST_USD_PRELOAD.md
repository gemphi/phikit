# Step-by-Step: Botanist USD/USDZ Preload

> This guide explains exactly how Botanist now preloads USD assets and how to verify it is working.

---

## 📊 Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| **Manifest Loading** | ✅ Working | Asset manifest scan at startup |
| **OBJ Preload** | ✅ Working | Automatic OBJ file loading |
| **Texture Preload** | ✅ Working | PNG/JPG materials loaded |
| **USD Scan** | ✅ Working | Detects .usd/.usda/.usdc/.usdz |
| **USD Preload** | ⚠️ Partial | Basic structure parsed |
| **USD Runtime Load** | ✅ Working | Load USD at runtime |
| **Audio Preload** | ✅ Working | Audio files in manifest |
| **Material Preload** | ⚠️ Partial | Standard materials only |
| **Full USD Parser** | ⚠️ Partial | Primitives supported |

**Last Updated:** 2024-01-20  
**App Status:** ✅ Running & Ready  
**Files:** `botanist/mod.rs`, `botanist/assets.rs`

---

## What Was Fixed

Botanist now does two startup asset passes:

1. Loads manifest assets (`OBJ`, textures, materials, audio).
2. Scans active Botanist assets and preloads all `.usd`, `.usda`, `.usdc`, and `.usdz` files.

The preload runs in:

- `app/phipro/src/projects/botanist/mod.rs`
- `app/phipro/src/projects/botanist/assets.rs`

---

## Step 1: Build Check

From the repository root:

```bash
cargo check -p phipro
```

Expected: build completes (warnings are acceptable if unrelated).

---

## Step 2: Run Botanist

From the repository root:

```bash
cargo run -p phipro -- botanist
```

---

## Step 3: Verify Startup Logs

During startup, confirm these lines appear:

- `Loading manifest assets...`
- `Manifest assets loaded: X/Y`
- `Preloading BOTanist USD/USDZ assets...`
- `USD assets loaded: A/B`

If `B > 0`, Botanist discovered USD-family files.
If `A > 0`, Botanist successfully read one or more USD-family files into the asset manager.

---

## Step 4: Where USD Files Are Scanned

The preload scans this active directory (relative to repo root):

- `assets/botanist`

Recommended structure:

- `assets/botanist/usd/...`

### One-Time Migration (if your USD files are still in old BOTanist docs)

Run from repo root in PowerShell:

```powershell
$dest = 'assets/botanist/usd'
$roots = @(
  'old-folder/phipro/doc/realitykit-pro/projects/BOTAnist/Packages/BotanistAssets/Sources/BOTanistAssets/BOTAnistAssets.rkassets/Assets',
  'old-folder/app/phipro/doc/realitykit-pro/projects/BOTAnist/Packages/BotanistAssets/Sources/BOTanistAssets/BOTAnistAssets.rkassets/Assets'
)

foreach ($root in $roots) {
  $resolved = (Resolve-Path $root).Path
  Get-ChildItem -Path $resolved -Recurse -File |
    Where-Object { $_.Extension -match '^\.(usd|usda|usdc|usdz)$' } |
    ForEach-Object {
      $rel = $_.FullName.Substring($resolved.Length).TrimStart('\\')
      $target = Join-Path $dest $rel
      New-Item -ItemType Directory -Force -Path (Split-Path $target) | Out-Null
      Copy-Item -Path $_.FullName -Destination $target -Force
    }
}
```

---

## Step 5: Add Your Own USD/USDZ Files

Place files under active assets, for example:

1. `assets/botanist/usd/`
2. `assets/botanist/meshes/`

Supported extensions are case-insensitive:

- `.usd`
- `.usda`
- `.usdc`
- `.usdz`

Restart Botanist after adding files.

---

## Troubleshooting

If you still cannot load USD assets:

1. Confirm file extension is exactly one of: `usd/usda/usdc/usdz`.
2. Confirm file path is under `assets/botanist`.
3. Re-run and check `USD assets loaded: A/B`.
4. If `B` is `0`, the scanner did not find files in configured roots.
5. If `B > 0` but `A` is `0`, files were found but could not be read (permissions or invalid path).

---

## Code References

- `app/phipro/src/projects/botanist/assets.rs`
- `app/phipro/src/projects/botanist/mod.rs`
