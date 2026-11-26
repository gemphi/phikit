//! A USD layer as text (`.usda`): what it names of other files (its
//! references, payloads, textures, each between `@`), each put where a
//! package keeps it, and its root turned about the up axis. Only what a
//! package needs; a layer is read nowhere else here.

/// Every file a layer names, as it names them, once each, in order.
pub fn named(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find('@') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('@') else { break };
        let name = &after[..close];
        // `@@` and a line's own `@` are not a name; a name is kept once.
        let fresh = !name.is_empty() && !name.contains('\n') && !out.iter().any(|n| n == name);
        fresh.then(|| out.push(name.to_string()));
        rest = &after[close + 1..];
    }
    out
}

/// The layer with each name it gives put as another, where one is given.
pub fn renamed(text: &str, to: &[(String, String)]) -> String {
    to.iter().fold(text.to_string(), |text, (from, into)| text.replace(&format!("@{from}@"), &format!("@{into}@")))
}

/// The layer with its root (its `defaultPrim`) turned about the up axis
/// by `yaw` radians: its front, which faced the eye, now faces +z, where
/// a viewer of it looks from. A root of its own transforms keeps them,
/// the turn taken after.
pub fn turned(text: &str, yaw: f32) -> String {
    let Some(root) = default_prim(text) else { return text.to_string() };
    let Some(open) = root_block(text, &root) else { return text.to_string() };
    let (sin, cos) = (-yaw).sin_cos();
    let matrix = format!("matrix4d xformOp:transform:turn = ( ({cos}, 0, {}, 0), (0, 1, 0, 0), ({sin}, 0, {cos}, 0), (0, 0, 0, 1) )", -sin);
    let body = &text[open..];
    let end = block_end(body).unwrap_or(body.len());
    // Its own order at its own depth: the turn goes first, the outermost.
    let own = body[..end].lines().scan(0i32, |depth, line| {
        let here = *depth;
        *depth += line.matches('{').count() as i32 - line.matches('}').count() as i32;
        Some((here, line))
    }).find(|(depth, line)| *depth == 1 && line.trim_start().starts_with("uniform token[] xformOpOrder"));
    let indent = "    ".repeat(root_depth(text, open) + 1);
    match own {
        Some((_, line)) => {
            let ordered = line.replacen("= [", "= [\"xformOp:transform:turn\", ", 1);
            let at = open + body.find(line).unwrap_or(0);
            format!("{}{indent}{matrix}\n{ordered}{}", &text[..at], &text[at + line.len()..])
        }
        None => {
            let line = format!("\n{indent}{matrix}\n{indent}uniform token[] xformOpOrder = [\"xformOp:transform:turn\"]");
            format!("{}{line}{}", &text[..open + 1], &text[open + 1..])
        }
    }
}

/// The name of the layer's root.
fn default_prim(text: &str) -> Option<String> {
    let line = text.lines().take_while(|line| !line.trim_start().starts_with("def ")).find(|line| line.trim_start().starts_with("defaultPrim"))?;
    Some(line.split('"').nth(1)?.to_string())
}

/// Where the root's block opens: the `{` after its `def`.
fn root_block(text: &str, root: &str) -> Option<usize> {
    let def = text.lines().scan(0usize, |at, line| {
        let here = *at;
        *at += line.len() + 1;
        Some((here, line))
    }).find(|(_, line)| line.trim_start().starts_with("def ") && line.contains(&format!("\"{root}\"")))?.0;
    Some(def + text[def..].find('{')?)
}

/// How deep the root stands: the blocks open before it.
fn root_depth(text: &str, open: usize) -> usize {
    let before = &text[..open];
    before.matches('{').count().saturating_sub(before.matches('}').count())
}

/// Where a block that opens at the start of `body` closes.
fn block_end(body: &str) -> Option<usize> {
    let mut depth = 0i32;
    for (at, c) in body.char_indices() {
        depth += match c {
            '{' => 1,
            '}' => -1,
            _ => 0,
        };
        match depth {
            0 if c == '}' => return Some(at + 1),
            _ => {}
        }
    }
    None
}
