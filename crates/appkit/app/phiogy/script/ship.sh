#!/bin/bash
# Send what the line's Linux build needs to a server: phikit cut down to
# the crates `phiogy` builds, with a workspace file naming only them.
# The server then builds the image of ../Dockerfile.
#
#   bash crates/appkit/app/phiogy/script/ship.sh ubuntu@HOST KEY [FOLDER]
#
# FOLDER is where it lands on the server (default gemphi/phikit). What
# was there before is moved to deploy-backups; nothing else is touched.
set -euo pipefail
to="$1"; key="$2"; folder="${3:-gemphi/phikit}"
cd "$(dirname "$0")/../../../../.."
stage="$(mktemp -d)"
trap 'rm -rf "$stage"' EXIT

# The crates of this workspace `phiogy` names on any system and for its
# tests: cargo reads the file of each before it builds the ones Linux
# needs.
cargo tree -p phiogy --offline --target all -e all --prefix none \
  | grep -o '(.:\\[^)]*)' | tr -d '()' | sort -u > "$stage/crate.txt"
python - "$stage" <<'PY'
import os, re, sys
stage = sys.argv[1]
root = os.getcwd()
crate = [os.path.relpath(line.strip(), root).replace(os.sep, '/') for line in open(os.path.join(stage, 'crate.txt')) if line.strip()]
text = open('Cargo.toml', encoding='utf-8').read()
member = 'members = [\n' + ''.join(f'    "{one}",\n' for one in sorted(crate)) + ']\n'
text = re.sub(r'members = \[.*?\n\]\n', lambda _: member, text, count=1, flags=re.S)
text = re.sub(r'default-members = .*\n', '', text, count=1)
open(os.path.join(stage, 'Cargo.toml'), 'w', encoding='utf-8', newline='\n').write(text)
open(os.path.join(stage, 'crate.txt'), 'w', newline='\n').write('\n'.join(sorted(crate)) + '\n')
PY
echo "crates: $(wc -l < "$stage/crate.txt")"

# A crate that lies inside another (the GPU crates inside gpui, Phiogy
# inside appkit) is sent with it; the other apps of appkit are not. The
# core folder is sent whole: a crate there may be named by a feature
# nothing turns on, and cargo still reads its file.
top="$(awk 'NR==1 || index($0, last "/") != 1 { print; last = $0 }' "$stage/crate.txt")"
tar -czf "$stage/src.tgz" --exclude=target --exclude=reference --exclude=old --exclude=doc \
  --exclude=web/pkg --exclude='crates/appkit/app/phigen' \
  Cargo.lock .cargo/config.toml crates/phicore $(echo "$top" | grep -v '^crates/phicore/')
ls -la "$stage/src.tgz" | awk '{ print "sending", $5, "bytes" }'
ssh -i "$key" -o BatchMode=yes "$to" "mkdir -p deploy-backups && { [ ! -e '$folder' ] || mv '$folder' deploy-backups/phikit-\$(date +%Y%m%d-%H%M%S); } && mkdir -p '$folder'"
cat "$stage/src.tgz" | ssh -i "$key" -o BatchMode=yes "$to" "tar -xzf - -C '$folder'"
scp -q -i "$key" -o BatchMode=yes "$stage/Cargo.toml" "$to:$folder/Cargo.toml"
ssh -i "$key" -o BatchMode=yes "$to" "cd '$folder' && du -sh . && ls"
