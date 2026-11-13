# fan2jian 繁→简

[![Crates.io](https://img.shields.io/crates/v/fan2jian.svg)](https://crates.io/crates/fan2jian)
[![docs.rs](https://img.shields.io/docsrs/fan2jian)](https://docs.rs/fan2jian)
[![Téléchargements](https://img.shields.io/crates/d/fan2jian.svg)](https://crates.io/crates/fan2jian)
[![MSRV](https://img.shields.io/crates/msrv/fan2jian.svg)](https://crates.io/crates/fan2jian)
[![Licence](https://img.shields.io/crates/l/fan2jian.svg)](https://crates.io/crates/fan2jian)
[![Dépendances](https://img.shields.io/badge/dépendances-0-brightgreen.svg)](https://crates.io/crates/fan2jian/dependencies)
[![unsafe interdit](https://img.shields.io/badge/unsafe-interdit-success.svg)](https://github.com/rust-secure-code/safety-dance/)

Convertisseur **chinois traditionnel → chinois simplifié**, ultra-rapide, **sans aucune dépendance**.

```rust
let t = "企畫 計畫 企劃 計劃 畫圖 畫畫";
assert_eq!(fan2jian::convert(t), "企划 计划 企划 计划 画图 画画");
```

## Pourquoi

- **Zéro dépendance** : tables générées à la compilation par `build.rs` (std uniquement).
- **O(1) par caractère** : page table à deux niveaux (pas de hachage).
- **Expressions d'abord** : correspondance la plus longue (dictionnaire d'expressions OpenCC).
- **Zéro-copie** : `convert` renvoie un `Cow::Borrowed` quand il n'y a rien à convertir
  (texte anglais, déjà simplifié…).
- **100 % safe** : `#![forbid(unsafe_code)]`.

## API

| Fonction | Rôle |
|---|---|
| `convert(&str) -> Cow<str>` | conversion, sans allocation si rien ne change |
| `convert_into(&str, &mut String)` | ajoute le résultat à un tampon réutilisable |
| `convert_in_place(&mut String)` | convertit sur place |
| `convert_char(char) -> char` | un seul caractère, hors contexte |
| `contains_traditional(&str) -> bool` | détection |

## CLI

```bash
cargo install --path .
echo "計劃" | f2j          # -> 计划
f2j fichier.txt > out.txt
```

Exemple d'usage de la lib : `cargo run --example basic`.

## Données et extension

- `data/TSCharacters.txt`, `data/TSPhrases.txt` : dictionnaires [OpenCC](https://github.com/BYVoid/OpenCC) (Apache-2.0, voir `NOTICE`).
- `data/ExtraPhrases.txt` : vos propres expressions (`clé<TAB>valeur`), prioritaires sur OpenCC.
  Modifiez-le, relancez `cargo build` : les tables sont régénérées.

## Limites

La conversion est faite par caractères et expressions, sans analyse sémantique ni segmentation
de mots. Elle ne convertit pas les variantes lexicales régionales (Taïwan / Hong Kong → vocabulaire
du continent, ex. 軟體 → 软件) : ce serait l'objet de dictionnaires supplémentaires.

## Licence

GPL-2.0-or-later pour le code. Les dictionnaires OpenCC restent sous Apache-2.0.

Copyright (C) 2026 Jorge Andre Castro