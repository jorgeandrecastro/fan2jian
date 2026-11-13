use fan2jian::{contains_traditional, convert, convert_in_place, convert_into};

fn main() {
    let t = "企畫 計畫 企劃 計劃 畫圖 畫畫";
    println!("{t}  ->  {}", convert(t));

    let mut buf = String::new();
    convert_into("頭髮 麵包", &mut buf);
    println!("convert_into : {buf}");

    let mut s = String::from("繁體中文");
    convert_in_place(&mut s);
    println!("in place     : {s}");

    println!("contient du traditionnel ? {}", contains_traditional("简体"));
}
