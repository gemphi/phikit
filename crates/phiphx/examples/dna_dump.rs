//! Print DNA struct layouts: `cargo run --example dna_dump -- file.blend Struct...`
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let f = phiphx::blend::BlendFile::parse_bytes(&std::fs::read(&a[1]).unwrap()).unwrap();
    let dna = f.dna.as_ref().unwrap();
    for want in &a[2..] {
        for s in dna.structs.iter().filter(|s| &dna.types[s.type_index as usize] == want) {
            let fields: Vec<String> = s.fields.iter().map(|x| format!("{} {}", dna.types[x.type_index as usize], dna.names[x.name_index as usize])).collect();
            println!("{want}: {}", fields.join("; "));
        }
    }
}
