$ErrorActionPreference = 'Stop'
$enc = New-Object System.Text.UTF8Encoding($false)
Set-Location C:\Users\phiac\Workspace\gemphi\_temp\phikit-usd\crates
$blend = 'phiphx/src/device/blend'

# (dir under device/blend, old file stem, new file stem, parent module name)
$Ren = @(
 @('anim','channels','channel','anim'),
 @('anim','drivers','driver_list','anim'),
 @('anim','handles','handle','anim'),
 @('file','names','name','file'),
 @('file','readers','reader','file'),
 @('file','types','kind','file'),
 @('read/curve','lists','list','curve'),
 @('read/mesh','attrs','attr','mesh'),
 @('read/mesh','faces','face','mesh'),
 @('read/mesh','groups','group','mesh'),
 @('read/mesh','normals','normal','mesh'),
 @('read/mesh','offsets','offset','mesh'),
 @('read/mesh','positions','position','mesh'),
 @('modifier','merge_verts','merge_vertex','modifier'),
 @('modifier','nodes','node','modifier'),
 @('modifier','nodes_corners','node_corner','modifier'),
 @('modifier','nodes_fans','node_fan','modifier'),
 @('modifier','nodes_field','node_field','modifier'),
 @('modifier','nodes_inputs','node_input','modifier'),
 @('modifier','nodes_legacy','node_legacy','modifier'),
 @('modifier','nodes_ops','node_op','modifier'),
 @('modifier','nodes_topo','node_topo','modifier'),
 @('modifier','solidify_creases','solidify_crease','modifier'),
 @('modifier','solidify_normals','solidify_normal','modifier'),
 @('read/object','instance_members','instance_member','object'),
 @('read/scene','props','prop','scene'),
 @('read/scene','types','kind','scene'),
 @('shade','links','link','shade'),
 @('shade','maps','map','shade'),
 @('shade','nodes','node','shade'),
 @('shade','slots','slot','shade'),
 @('read/text','chars','character','text'),
 @('read/text/font','names','name','font'),
 @('write','keys','key','write'),
 @('write/script','chunks','chunk','script'),
 @('write/script','helpers','helper','script'),
 @('write/script','types','kind','script')
)

# mod declarations: in the dir's mod.rs
foreach ($e in $Ren) {
    $dir, $old, $new, $parent = $e
    $modrs = "$blend/$dir/mod.rs"
    if ($dir -eq 'write/script') { $modrs = "$blend/$dir/mod.rs" }
    $t = [IO.File]::ReadAllText((Resolve-Path $modrs).Path)
    $n = [regex]::Replace($t, "(?m)^(\s*(?:pub(?:\([a-z]+\))? )?mod )$old;", "`${1}$new;")
    if ($n -eq $t) { Write-Host "NO mod decl for $dir/$old in $modrs" }
    [IO.File]::WriteAllText((Resolve-Path $modrs).Path, $n, $enc)
}

# qualified paths everywhere: parent::old -> parent::new
$files = git ls-files -- phiphx phiber | Where-Object { $_ -like '*.rs' }
foreach ($f in $files) {
    $full = (Resolve-Path $f).Path
    $t = [IO.File]::ReadAllText($full); $o = $t
    foreach ($e in $Ren) {
        $dir, $old, $new, $parent = $e
        $t = [regex]::Replace($t, "\b$parent::$old\b", "$parent::$new")
    }
    if ($t -ne $o) { [IO.File]::WriteAllText($full, $t, $enc) }
}
