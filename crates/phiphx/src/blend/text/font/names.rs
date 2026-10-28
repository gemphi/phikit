//! Glyph name → Unicode, the part of the Adobe Glyph List a Type 1 font's
//! Unicode charmap is synthesized from (FreeType `psnames`,
//! `ps_unicode_value`): `uniXXXX` / `uXXXX[XX]` names, then the list, for
//! every name `bfont.pfb` carries (ASCII, Latin-1 and the
//! StandardEncoding extras).

/// The Unicode value FreeType gives glyph `name`, `None` for unknown names.
pub fn unicode_of(name: &str) -> Option<u32> {
    let base = name.split('.').next().unwrap_or(name);
    if let Some(hex) = base.strip_prefix("uni").filter(|h| h.len() == 4) {
        return u32::from_str_radix(hex, 16).ok();
    }
    if let Some(hex) = base.strip_prefix('u').filter(|h| (4..=6).contains(&h.len())) {
        if let Ok(v) = u32::from_str_radix(hex, 16) {
            return Some(v);
        }
    }
    if base.len() == 1 && base.as_bytes()[0].is_ascii_alphabetic() {
        return Some(base.as_bytes()[0] as u32);
    }
    // `name.suffix` variants are not mapped (`ps_unicode_value` gives them
    // the variant flag, which a charmap lookup skips).
    match name.contains('.') && name != ".notdef" {
        true => None,
        false => AGL.iter().find(|(n, _)| *n == base).map(|&(_, u)| u),
    }
}

/// The glyph name at `code` in Adobe StandardEncoding: printable ASCII
/// (with its curly quotes) and the accents `seac` composes with.
pub fn standard_name(code: u8) -> Option<&'static str> {
    const ASCII: [&str; 95] = [
        "space", "exclam", "quotedbl", "numbersign", "dollar", "percent", "ampersand", "quoteright", "parenleft", "parenright", "asterisk", "plus", "comma", "hyphen", "period", "slash",
        "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "colon", "semicolon", "less", "equal", "greater", "question", "at", "A", "B", "C", "D", "E", "F", "G",
        "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "bracketleft", "backslash", "bracketright", "asciicircum", "underscore",
        "quoteleft", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "braceleft", "bar", "braceright", "asciitilde",
    ];
    match code {
        32..=126 => Some(ASCII[(code - 32) as usize]),
        193 => Some("grave"),
        194 => Some("acute"),
        195 => Some("circumflex"),
        196 => Some("tilde"),
        197 => Some("macron"),
        198 => Some("breve"),
        199 => Some("dotaccent"),
        200 => Some("dieresis"),
        202 => Some("ring"),
        203 => Some("cedilla"),
        205 => Some("hungarumlaut"),
        206 => Some("ogonek"),
        207 => Some("caron"),
        245 => Some("dotlessi"),
        _ => None,
    }
}

const AGL: &[(&str, u32)] = &[
    ("space", 0x20), ("exclam", 0x21), ("quotedbl", 0x22), ("numbersign", 0x23), ("dollar", 0x24),
    ("percent", 0x25), ("ampersand", 0x26), ("quotesingle", 0x27), ("parenleft", 0x28), ("parenright", 0x29),
    ("asterisk", 0x2A), ("plus", 0x2B), ("comma", 0x2C), ("hyphen", 0x2D), ("period", 0x2E), ("slash", 0x2F),
    ("zero", 0x30), ("one", 0x31), ("two", 0x32), ("three", 0x33), ("four", 0x34), ("five", 0x35),
    ("six", 0x36), ("seven", 0x37), ("eight", 0x38), ("nine", 0x39), ("colon", 0x3A), ("semicolon", 0x3B),
    ("less", 0x3C), ("equal", 0x3D), ("greater", 0x3E), ("question", 0x3F), ("at", 0x40),
    ("bracketleft", 0x5B), ("backslash", 0x5C), ("bracketright", 0x5D), ("asciicircum", 0x5E),
    ("underscore", 0x5F), ("grave", 0x60), ("braceleft", 0x7B), ("bar", 0x7C), ("braceright", 0x7D),
    ("asciitilde", 0x7E), ("exclamdown", 0xA1), ("cent", 0xA2), ("sterling", 0xA3), ("currency", 0xA4),
    ("yen", 0xA5), ("brokenbar", 0xA6), ("section", 0xA7), ("dieresis", 0xA8), ("copyright", 0xA9),
    ("ordfeminine", 0xAA), ("guillemotleft", 0xAB), ("logicalnot", 0xAC), ("registered", 0xAE),
    ("macron", 0xAF), ("degree", 0xB0), ("plusminus", 0xB1), ("twosuperior", 0xB2), ("threesuperior", 0xB3),
    ("acute", 0xB4), ("mu", 0xB5), ("paragraph", 0xB6), ("periodcentered", 0xB7), ("cedilla", 0xB8),
    ("onesuperior", 0xB9), ("ordmasculine", 0xBA), ("guillemotright", 0xBB), ("onequarter", 0xBC),
    ("onehalf", 0xBD), ("threequarters", 0xBE), ("questiondown", 0xBF), ("Agrave", 0xC0), ("Aacute", 0xC1),
    ("Acircumflex", 0xC2), ("Atilde", 0xC3), ("Adieresis", 0xC4), ("Aring", 0xC5), ("AE", 0xC6),
    ("Ccedilla", 0xC7), ("Egrave", 0xC8), ("Eacute", 0xC9), ("Ecircumflex", 0xCA), ("Edieresis", 0xCB),
    ("Igrave", 0xCC), ("Iacute", 0xCD), ("Icircumflex", 0xCE), ("Idieresis", 0xCF), ("Eth", 0xD0),
    ("Ntilde", 0xD1), ("Ograve", 0xD2), ("Oacute", 0xD3), ("Ocircumflex", 0xD4), ("Otilde", 0xD5),
    ("Odieresis", 0xD6), ("multiply", 0xD7), ("Oslash", 0xD8), ("Ugrave", 0xD9), ("Uacute", 0xDA),
    ("Ucircumflex", 0xDB), ("Udieresis", 0xDC), ("Yacute", 0xDD), ("Thorn", 0xDE), ("germandbls", 0xDF),
    ("agrave", 0xE0), ("aacute", 0xE1), ("acircumflex", 0xE2), ("atilde", 0xE3), ("adieresis", 0xE4),
    ("aring", 0xE5), ("ae", 0xE6), ("ccedilla", 0xE7), ("egrave", 0xE8), ("eacute", 0xE9),
    ("ecircumflex", 0xEA), ("edieresis", 0xEB), ("igrave", 0xEC), ("iacute", 0xED), ("icircumflex", 0xEE),
    ("idieresis", 0xEF), ("eth", 0xF0), ("ntilde", 0xF1), ("ograve", 0xF2), ("oacute", 0xF3),
    ("ocircumflex", 0xF4), ("otilde", 0xF5), ("odieresis", 0xF6), ("divide", 0xF7), ("oslash", 0xF8),
    ("ugrave", 0xF9), ("uacute", 0xFA), ("ucircumflex", 0xFB), ("udieresis", 0xFC), ("yacute", 0xFD),
    ("thorn", 0xFE), ("ydieresis", 0xFF), ("dotlessi", 0x131), ("Lslash", 0x141), ("lslash", 0x142),
    ("OE", 0x152), ("oe", 0x153), ("Scaron", 0x160), ("scaron", 0x161), ("Ydieresis", 0x178),
    ("Zcaron", 0x17D), ("zcaron", 0x17E), ("florin", 0x192), ("circumflex", 0x2C6), ("caron", 0x2C7),
    ("breve", 0x2D8), ("dotaccent", 0x2D9), ("ring", 0x2DA), ("ogonek", 0x2DB), ("tilde", 0x2DC),
    ("hungarumlaut", 0x2DD), ("endash", 0x2013), ("emdash", 0x2014), ("quoteleft", 0x2018),
    ("quoteright", 0x2019), ("quotesinglbase", 0x201A), ("quotedblleft", 0x201C), ("quotedblright", 0x201D),
    ("quotedblbase", 0x201E), ("dagger", 0x2020), ("daggerdbl", 0x2021), ("bullet", 0x2022),
    ("ellipsis", 0x2026), ("perthousand", 0x2030), ("guilsinglleft", 0x2039), ("guilsinglright", 0x203A),
    ("fraction", 0x2044), ("trademark", 0x2122), ("minus", 0x2212), ("fi", 0xFB01), ("fl", 0xFB02),
];
