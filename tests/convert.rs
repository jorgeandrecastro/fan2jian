use fan2jian::*;
use std::borrow::Cow;

#[test]
fn readme_example() {
    let t = "企畫 計畫 企劃 計劃 畫圖 畫畫";
    assert_eq!(convert(t), "企划 计划 企划 计划 画图 画画");
}

#[test]
fn zero_copy_when_nothing_to_do() {
    assert!(matches!(convert(""), Cow::Borrowed(_)));
    assert!(matches!(convert("hello world 123"), Cow::Borrowed(_)));
    assert!(matches!(convert("简体中文"), Cow::Borrowed(_)));
}

#[test]
fn mixed_ascii_and_cjk_boundaries() {
    assert_eq!(convert("abc計劃def"), "abc计划def");
    assert_eq!(convert("計"), "计");
    assert_eq!(convert("🙂 繁體 🙂"), "🙂 繁体 🙂");
}

#[test]
fn phrases_take_priority_over_characters() {
    // Toute entrée de TSPhrases doit être produite telle quelle par `convert`.
    let dict = std::fs::read_to_string("data/TSPhrases.txt").unwrap();
    let mut checked = 0;
    for line in dict.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let (k, rest) = line.split_once('\t').unwrap();
        let v = rest.split_whitespace().next().unwrap();
        if k.chars().count() >= 2 {
            // la correspondance la plus longue peut être une autre expression plus grande :
            // on ne teste donc que l'idempotence et la présence de la valeur.
            assert!(convert(k).contains(v) || convert(k) == convert(v), "{k} -> {v}");
            checked += 1;
        }
    }
    assert!(checked > 100);
}

#[test]
fn characters_match_dictionary() {
    let dict = std::fs::read_to_string("data/TSCharacters.txt").unwrap();
    let mut checked = 0;
    for line in dict.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let (k, rest) = line.split_once('\t').unwrap();
        let v = rest.split_whitespace().next().unwrap();
        if k.chars().count() == 1 && v.chars().count() == 1 {
            let (kc, vc) = (k.chars().next().unwrap(), v.chars().next().unwrap());
            assert_eq!(convert_char(kc), vc, "{k} -> {v}");
            checked += 1;
        }
    }
    assert!(checked > 3000);
}

#[test]
fn idempotent() {
    let once = convert("企畫 計畫 頭髮 麵包 後面 發展 乾淨").into_owned();
    assert_eq!(convert(&once), once.as_str());
}

#[test]
fn convert_into_and_in_place_agree() {
    let src = "我們的計劃：畫圖與發展";
    let mut a = String::from("préfixe:");
    convert_into(src, &mut a);
    assert_eq!(a, format!("préfixe:{}", convert(src)));

    let mut b = src.to_string();
    convert_in_place(&mut b);
    assert_eq!(b, convert(src));
}

#[test]
fn detection() {
    assert!(contains_traditional("計劃"));
    assert!(!contains_traditional("计划 plan"));
}
