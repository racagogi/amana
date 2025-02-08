use std::{
    fs::{metadata, File},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::exit,
};

use clap::{value_parser, Arg, Command, ValueEnum};
use nougui::diceware;
use rand::{rngs::StdRng, Rng, SeedableRng};

fn read_file(path: &Path) -> Vec<String> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => {
            eprintln!("Can not open file {:?}", path);
            exit(1);
        }
    };

    let metadata = match metadata(path) {
        Ok(m) => m,
        Err(_) => {
            eprintln!("Can not read metadata of file {:?}", path);
            exit(1);
        }
    };

    if !metadata.is_file() {
        eprintln!("The path {:?} is not a file.", path);
        exit(1);
    }

    let reader = BufReader::new(file);
    make_wordlist(reader.lines().filter_map(|x| x.ok())).collect()
}

fn read_string(contents: &str) -> Vec<String> {
    make_wordlist(contents.lines().map(|s| s.to_string())).collect()
}

fn make_wordlist<I>(lines: I) -> impl Iterator<Item = String>
where
    I: Iterator<Item = String>,
{
    lines.map(|x| {
        let mut words = x.trim().split_whitespace();
        match (words.next(), words.next()) {
            (_, Some(w)) => w.to_string(),
            (Some(w), _) => w.to_string(),
            _ => "".to_string(),
        }
    })
}

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
enum BuiltinFile {
    EffLarge,
    EffShort1,
    EffShort2,
}

fn cli() -> Command {
    Command::new("diceware")
        .about("Generate secure passphrases using diceware")
        .author("racagogi")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_parser(value_parser!(PathBuf))
                .num_args(1)
                .help("Word list path; if not given, uses the default wordlist"),
        )
        .arg(
            Arg::new("builtin")
                .short('b')
                .action(clap::ArgAction::Set)
                .value_parser(clap::value_parser!(BuiltinFile))
                .num_args(1)
                .conflicts_with("input"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_parser(value_parser!(PathBuf))
                .num_args(1)
                .help("Passphrase output path"),
        )
        .arg(
            Arg::new("count")
                .short('c')
                .long("count")
                .value_parser(value_parser!(u8))
                .default_value("4")
                .num_args(1)
                .help("Number of words in the passphrase"),
        )
        .arg(
            Arg::new("delimiter")
                .short('d')
                .long("delimiter")
                .value_parser(value_parser!(char))
                .default_value("-")
                .num_args(1)
                .help("Delimiter between words in the passphrase"),
        )
        .arg(
            Arg::new("numbers")
                .short('n')
                .long("numbers")
                .value_parser(value_parser!(u8))
                .default_value("1")
                .num_args(1)
                .help("Number of passphrases to generate"),
        )
}

fn main() {
    let mathes = cli().get_matches();

    let input = mathes.get_one::<PathBuf>("input");
    let builtin = mathes.get_one::<BuiltinFile>("builtin");
    let output = mathes.get_one::<PathBuf>("output");
    let count = *mathes.get_one::<u8>("count").unwrap();
    let numbers = *mathes.get_one::<u8>("numbers").unwrap();
    let delimiter = mathes.get_one::<char>("delimiter").unwrap();

    let eff_large = include_str!("../../source/eff_large_wordlist.txt");
    let eff_short1 = include_str!("../../source/eff_short_wordlist_1.txt");
    let eff_short2 = include_str!("../../source/eff_short_wordlist_2_0.txt");

    let words = match builtin {
        Some(BuiltinFile::EffLarge) => read_string(&eff_large),
        Some(BuiltinFile::EffShort1) => read_string(&eff_short1),
        Some(BuiltinFile::EffShort2) => read_string(&eff_short2),
        None => match input {
            Some(p) => read_file(p),
            None => {
                let mut rnd = StdRng::from_os_rng();
                let x: u32 = rnd.random();
                match x % 3 {
                    0 => read_string(&eff_large),
                    1 => read_string(&eff_short1),
                    2 => read_string(&eff_short2),
                    _ => read_string(&eff_large),
                }
            }
        },
    };

    match output {
        Some(p) => {
            if let Ok(mut file) = File::options().append(true).create(true).open(p) {
                for _ in 0..numbers {
                    let passwd = diceware::select_words(&words, count, &delimiter.to_string());
                    let _ = file.write(passwd.as_bytes());
                    let _ = file.write("\n".as_bytes());
                }
            } else {
                eprintln!("Can not open file {:?}", p);
                exit(1);
            }
        }
        None => {
            for _ in 0..numbers {
                let passwd = diceware::select_words(&words, count, &delimiter.to_string());
                println!("{passwd}");
            }
        }
    }
}
