//! # fan2jian
//!
//! Convertisseur **chinois traditionnel (繁体) → chinois simplifié (简体)**, sans aucune
//! dépendance.
//!
//! * Tables OpenCC (`TSCharacters` + `TSPhrases`) compilées dans le binaire par `build.rs`.
//! * Caractères : page table à deux niveaux, accès en O(1), sans hachage.
//! * Expressions : correspondance la plus longue d'abord (gère les cas dépendant du contexte).
//! * Zéro-copie : si le texte ne contient rien à convertir, [`convert`] renvoie un `Cow::Borrowed`.
//!
//! ```
//! let t = "企畫 計畫 企劃 計劃 畫圖 畫畫";
//! assert_eq!(fan2jian::convert(t), "企划 计划 企划 计划 画图 画画");
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::borrow::Cow;

#[allow(clippy::unreadable_literal)]
mod tables {
    include!(concat!(env!("OUT_DIR"), "/tables.rs"));
}
use tables::{CHAR1, CHAR2, GROUP_RANGES, HEAD1, HEAD2, PAGE_BITS, PHRASES};

const PAGE_MASK: u32 = (1 << PAGE_BITS) - 1;

/// Recherche O(1) dans une table à deux niveaux. Renvoie 0 si absent.
#[inline(always)]
fn lookup(stage1: &[u16], stage2: &[u32], cp: u32) -> u32 {
    match stage1.get((cp >> PAGE_BITS) as usize) {
        Some(&page) => stage2[((page as usize) << PAGE_BITS) | (cp & PAGE_MASK) as usize],
        None => 0,
    }
}

/// Résultat d'une correspondance à une position donnée.
enum Hit {
    Char(char),
    Phrase(&'static str),
}

/// Tente une conversion au début de `rest` (dont le premier caractère est `c`, non-ASCII).
#[inline]
fn convert_at(rest: &str, c: char) -> Option<(Hit, usize)> {
    let cp = c as u32;

    let group = lookup(&HEAD1, &HEAD2, cp);
    if group != 0 {
        let (start, end) = GROUP_RANGES[group as usize - 1];
        for &(from, to) in &PHRASES[start as usize..end as usize] {
            if rest.starts_with(from) {
                return Some((Hit::Phrase(to), from.len()));
            }
        }
    }

    let simp = lookup(&CHAR1, &CHAR2, cp);
    if simp != 0 {
        return Some((Hit::Char(char::from_u32(simp)?), c.len_utf8()));
    }
    None
}

/// Itérateur sur les positions (en octets) à réécrire.
struct Hits<'a> {
    input: &'a str,
    pos: usize,
}

impl Iterator for Hits<'_> {
    /// (début, correspondance, longueur consommée en octets)
    type Item = (usize, Hit, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let bytes = self.input.as_bytes();
        while self.pos < bytes.len() {
            // Chemin rapide : l'ASCII n'est jamais converti.
            if bytes[self.pos] < 0x80 {
                self.pos += 1;
                continue;
            }
            let rest = &self.input[self.pos..];
            let c = rest.chars().next()?;
            match convert_at(rest, c) {
                Some((hit, consumed)) => {
                    let start = self.pos;
                    self.pos += consumed;
                    return Some((start, hit, consumed));
                }
                None => self.pos += c.len_utf8(),
            }
        }
        None
    }
}

#[inline]
fn hits(input: &str) -> Hits<'_> {
    Hits { input, pos: 0 }
}

#[inline]
fn write_hit(out: &mut String, hit: &Hit) {
    match hit {
        Hit::Char(c) => out.push(*c),
        Hit::Phrase(to) => out.push_str(to),
    }
}

/// Convertit `input` en chinois simplifié.
///
/// Renvoie `Cow::Borrowed(input)` (aucune allocation) si rien n'est à convertir.
///
/// ```
/// use std::borrow::Cow;
/// assert!(matches!(fan2jian::convert("hello 你好"), Cow::Borrowed(_)));
/// assert_eq!(fan2jian::convert("頭髮"), "头发");
/// ```
pub fn convert(input: &str) -> Cow<'_, str> {
    let mut it = hits(input);
    let Some(first) = it.next() else {
        return Cow::Borrowed(input);
    };
    let mut out = String::with_capacity(input.len());
    let mut copied = 0;
    for (start, hit, consumed) in std::iter::once(first).chain(it) {
        out.push_str(&input[copied..start]);
        write_hit(&mut out, &hit);
        copied = start + consumed;
    }
    out.push_str(&input[copied..]);
    Cow::Owned(out)
}

/// Convertit `input` et **ajoute** le résultat à `out` (réutilisation de tampon, sans allocation
/// si `out` a déjà la capacité).
///
/// ```
/// let mut buf = String::new();
/// fan2jian::convert_into("計劃", &mut buf);
/// assert_eq!(buf, "计划");
/// ```
pub fn convert_into(input: &str, out: &mut String) {
    out.reserve(input.len());
    let mut copied = 0;
    for (start, hit, consumed) in hits(input) {
        out.push_str(&input[copied..start]);
        write_hit(out, &hit);
        copied = start + consumed;
    }
    out.push_str(&input[copied..]);
}

/// Convertit `s` sur place. Ne réalloue que si le texte change.
///
/// ```
/// let mut s = String::from("畫圖");
/// fan2jian::convert_in_place(&mut s);
/// assert_eq!(s, "画图");
/// ```
pub fn convert_in_place(s: &mut String) {
    let new = match convert(s) {
        Cow::Owned(o) => Some(o),
        Cow::Borrowed(_) => None,
    };
    if let Some(o) = new {
        *s = o;
    }
}

/// Convertit un caractère isolé (hors contexte). Renvoie `c` s'il n'a pas de forme simplifiée.
///
/// ```
/// assert_eq!(fan2jian::convert_char('畫'), '画');
/// assert_eq!(fan2jian::convert_char('a'), 'a');
/// ```
#[inline]
pub fn convert_char(c: char) -> char {
    match lookup(&CHAR1, &CHAR2, c as u32) {
        0 => c,
        s => char::from_u32(s).unwrap_or(c),
    }
}

/// `true` si `input` contient au moins un élément qui serait modifié par [`convert`].
///
/// ```
/// assert!(fan2jian::contains_traditional("計劃"));
/// assert!(!fan2jian::contains_traditional("计划 plan"));
/// ```
pub fn contains_traditional(input: &str) -> bool {
    hits(input).next().is_some()
}
