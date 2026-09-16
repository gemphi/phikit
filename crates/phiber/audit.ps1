$root = 'C:\Users\phiac\Workspace\gemphi\phiace\REFS\zed-repo\crates\phiber\src\v2'
Get-ChildItem -LiteralPath $root -Recurse -Filter '*.rs' | ForEach-Object {
    $c = (Get-Content -LiteralPath $_.FullName | Measure-Object -Line).Lines
    $rel = $_.FullName.Substring($root.Length + 1)
    [pscustomobject]@{ LOC = $c; File = $rel }
} | Where-Object { $_.LOC -gt 60 -and $_.File -notlike '*set.rs' } | Sort-Object -Property LOC -Descending
