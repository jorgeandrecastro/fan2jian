//! Génère `$OUT_DIR/tables.rs` à partir des dictionnaires OpenCC (`data/`).
//!
//! * `TSCharacters.txt` -> table caractère par caractère (page table à 2 niveaux, accès O(1)).
//! * `TSPhrases.txt`    -> expressions, indexées par premier caractère, triées du plus long au plus court.
//!
//! std uniquement, aucune dépendance.

use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

const PAGE_BITS: u32 = 8;
const PAGE_SIZE: usize = 1 << PAGE_BITS;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=data/TSCharacters.txt");
    println!("cargo:rerun-if-changed=data/TSPhrases.txt");
    println!("cargo:rerun-if-changed=data/ExtraPhrases.txt");

    let chars = parse_dict("data/TSCharacters.txt");
    // Les expressions complémentaires (ExtraPhrases) écrasent celles d'OpenCC.
    let mut phrases: BTreeMap<String, String> = parse_dict("data/TSPhrases.txt").into_iter().collect();
    phrases.extend(parse_dict("data/ExtraPhrases.txt"));

    // --- caractères : traditionnel -> simplifié (1 char -> 1 char) ---
    let mut char_map: BTreeMap<u32, u32> = BTreeMap::new();
    for (k, v) in &chars {
        let (mut kc, mut vc) = (k.chars(), v.chars());
        if let (Some(a), Some(b), None, None) = (kc.next(), vc.next(), kc.next(), vc.next()) {
            if a != b {
                char_map.insert(a as u32, b as u32);
            }
        }
    }

    // --- expressions, groupées par premier caractère ---
    let mut groups: BTreeMap<u32, Vec<(String, String)>> = BTreeMap::new();
    for (k, v) in phrases {
        if k.chars().count() >= 2 && k != v {
            let first = k.chars().next().unwrap() as u32;
            groups.entry(first).or_default().push((k, v));
        }
    }
    let mut flat: Vec<(String, String)> = Vec::new();
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    let mut head_map: BTreeMap<u32, u32> = BTreeMap::new();
    for (i, (first, mut list)) in groups.into_iter().enumerate() {
        // Deux clés qui matchent à la même position sont préfixes l'une de l'autre :
        // la plus longue en octets est donc la plus longue en caractères.
        list.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| a.0.cmp(&b.0)));
        let start = flat.len();
        flat.extend(list);
        ranges.push((start, flat.len()));
        head_map.insert(first, (i + 1) as u32); // 0 = « pas de groupe »
    }

    let mut out = String::new();
    writeln!(out, "// @generated par build.rs — ne pas modifier.").unwrap();
    writeln!(out, "pub const PAGE_BITS: u32 = {PAGE_BITS};").unwrap();
    two_level(&mut out, "CHAR", &char_map);
    two_level(&mut out, "HEAD", &head_map);

    writeln!(out, "pub static GROUP_RANGES: [(u32, u32); {}] = [", ranges.len()).unwrap();
    for (s, e) in &ranges {
        writeln!(out, "    ({s}, {e}),").unwrap();
    }
    writeln!(out, "];").unwrap();

    writeln!(out, "pub static PHRASES: [(&str, &str); {}] = [", flat.len()).unwrap();
    for (k, v) in &flat {
        writeln!(out, "    ({k:?}, {v:?}),").unwrap();
    }
    writeln!(out, "];").unwrap();

    let dest = Path::new(&env::var("OUT_DIR").unwrap()).join("tables.rs");
    fs::write(dest, out).unwrap();
}

/// Lit un dictionnaire OpenCC : `clé<TAB>valeur1 valeur2 ...`. On garde la 1re valeur.
fn parse_dict(path: &str) -> Vec<(String, String)> {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("lecture de {path}: {e}"));
    text.lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .filter_map(|l| {
            let (k, rest) = l.split_once('\t')?;
            let v = rest.split_whitespace().next()?;
            Some((k.to_string(), v.to_string()))
        })
        .collect()
}

/// Émet une table à deux niveaux `<NAME>1` (u16) / `<NAME>2` (u32), pages dédupliquées.
/// La page 0 est toujours la page vide ; la valeur 0 signifie « absent ».
fn two_level(out: &mut String, name: &str, map: &BTreeMap<u32, u32>) {
    let max = map.keys().next_back().copied().unwrap_or(0);
    let n_pages = (max >> PAGE_BITS) as usize + 1;
    let mut pages: Vec<Vec<u32>> = vec![vec![0; PAGE_SIZE]];
    let mut seen: HashMap<Vec<u32>, u16> = HashMap::new();
    seen.insert(pages[0].clone(), 0);
    let mut stage1 = vec![0u16; n_pages];

    for (p, slot) in stage1.iter_mut().enumerate() {
        let mut page = vec![0u32; PAGE_SIZE];
        for (i, cell) in page.iter_mut().enumerate() {
            if let Some(&v) = map.get(&(((p << PAGE_BITS) | i) as u32)) {
                *cell = v;
            }
        }
        *slot = *seen.entry(page.clone()).or_insert_with(|| {
            pages.push(page);
            (pages.len() - 1) as u16
        });
    }

    writeln!(out, "pub static {name}1: [u16; {}] = [", stage1.len()).unwrap();
    for chunk in stage1.chunks(16) {
        let line: Vec<String> = chunk.iter().map(u16::to_string).collect();
        writeln!(out, "    {},", line.join(", ")).unwrap();
    }
    writeln!(out, "];").unwrap();

    writeln!(out, "pub static {name}2: [u32; {}] = [", pages.len() * PAGE_SIZE).unwrap();
    for page in &pages {
        for chunk in page.chunks(16) {
            let line: Vec<String> = chunk.iter().map(u32::to_string).collect();
            writeln!(out, "    {},", line.join(", ")).unwrap();
        }
    }
    writeln!(out, "];").unwrap();
}
