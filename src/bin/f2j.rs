//! `f2j` : convertit du chinois traditionnel en simplifié.
//!
//! ```text
//! f2j fichier1.txt fichier2.txt   # fichiers -> stdout
//! echo "計劃" | f2j               # stdin -> stdout
//! ```

use std::io::{self, Read, Write};
use std::{env, fs, process};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("usage: f2j [FICHIER...]   (stdin si aucun fichier)");
        return;
    }

    let mut input = String::new();
    if args.is_empty() {
        if let Err(e) = io::stdin().read_to_string(&mut input) {
            eprintln!("f2j: stdin: {e}");
            process::exit(1);
        }
    } else {
        for path in &args {
            match fs::read_to_string(path) {
                Ok(t) => input.push_str(&t),
                Err(e) => {
                    eprintln!("f2j: {path}: {e}");
                    process::exit(1);
                }
            }
        }
    }

    let out = fan2jian::convert(&input);
    if io::stdout().lock().write_all(out.as_bytes()).is_err() {
        process::exit(1);
    }
}
