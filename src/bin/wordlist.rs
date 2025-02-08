use std::{
    collections::HashSet,
    fs::{metadata, File},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::exit,
};

use clap::{value_parser, Arg, Command, ValueEnum};
use nougui::hangul::keyborad::{self, dubeolsik_len, dubeolsik_word};
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
enum BuiltinFile {
    All,
    Cities,
    Colors,
    Countries,
    DayTime,
    Elements,
    Mountains,
    Numbers,
    Rivers,
    Spices,
    Treasures,
}

fn limit_len(word: &str, min: u8, max: u8, dubeolsik: bool) -> bool {
    if dubeolsik {
        let len = dubeolsik_len(word).try_into().unwrap();
        min <= len && len <= max
    } else {
        let len = word.chars().count().try_into().unwrap();
        min <= len && len <= max
    }
}

fn gen_dice_number(index: Vec<String>, dice: u8, len: usize) -> Vec<String> {
    if len <= 1 {
        index
    } else {
        let index = if index.is_empty() {
            (1..=dice).map(|x| x.to_string()).collect()
        } else {
            index
        };
        let nindex: Vec<String> = (1..=dice)
            .flat_map(|x| {
                let x = x.to_string();
                index
                    .iter()
                    .map(|i| format!("{}|{}", x, i))
                    .collect::<Vec<String>>()
            })
            .collect();

        gen_dice_number(nindex, dice, len - 1)
    }
}

fn filter_source<I>(words: I, min: u8, max: u8, dubeolsik: bool, filter_upper: bool) -> Vec<String>
where
    I: Iterator<Item = String>,
{
    words
        .filter(|x| {
            limit_len(x, min, max, dubeolsik)
                && (if filter_upper {
                    !keyborad::filter_shifted(x)
                } else {
                    true
                })
        })
        .map(|x| x.to_owned())
        .collect()
}

fn uniq_words(words: Vec<String>) -> Vec<String> {
    let set: HashSet<_> = words.into_iter().collect();
    set.into_iter().collect()
}

fn read_file(path: &Path, min: u8, max: u8, dubeolsik: bool, filter_upper: bool) -> Vec<String> {
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
    filter_source(
        reader.lines().filter_map(|l| l.ok()),
        min,
        max,
        dubeolsik,
        filter_upper,
    )
}

fn read_string(
    contents: &str,
    min: u8,
    max: u8,
    dubeolsik: bool,
    filter_upper: bool,
) -> Vec<String> {
    filter_source(
        contents.lines().map(|s| s.to_string()),
        min,
        max,
        dubeolsik,
        filter_upper,
    )
}

fn cli() -> Command {
    Command::new("wl")
        .about("Generate word list for dicewere")
        .author("racagogi")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_parser(value_parser!(PathBuf).clone())
                .action(clap::ArgAction::Append)
                .help("Word list path; if not given, uses the default wordlist"),
        )
        .arg(
            Arg::new("builtin")
                .short('b')
                .action(clap::ArgAction::Set)
                .value_parser(value_parser!(PathBuf))
                .num_args(1..)
                .value_parser(clap::value_parser!(BuiltinFile).clone()),
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
            Arg::new("dice")
                .short('d')
                .long("dice")
                .value_parser(value_parser!(u8))
                .default_value("6")
                .num_args(1)
                .help("Dice use for dicewere"),
        )
        .arg(
            Arg::new("number")
                .short('n')
                .long("number")
                .value_parser(value_parser!(u8))
                .default_value("4")
                .num_args(1)
                .help("Number of dice use for dicewere"),
        )
        .arg(
            Arg::new("min")
                .short('m')
                .long("min")
                .value_parser(value_parser!(u8))
                .default_value("1")
                .num_args(1)
                .help("Min length of a word"),
        )
        .arg(
            Arg::new("max")
                .short('M')
                .long("max")
                .value_parser(value_parser!(u8))
                .default_value("20")
                .num_args(1)
                .help("Max length of a word"),
        )
        .arg(
            Arg::new("filter_upper")
                .short('u')
                .action(clap::ArgAction::SetTrue)
                .default_value("false")
                .help("Remove uppercase jamo ㅃ,ㅉ,ㄸ,ㄲ,ㅆ,ㅒ,ㅖ"),
        )
        .arg(
            Arg::new("dubeolsik")
                .long("dubeol")
                .action(clap::ArgAction::SetTrue)
                .default_value("false")
                .help("use dubeolic encryption ㄱ-> r"),
        )
}

fn main() {
    let matches = cli().get_matches();

    let input = matches.get_one::<Vec<PathBuf>>("input");
    let builtin: Option<Vec<BuiltinFile>> = matches
        .get_many::<BuiltinFile>("builtin")
        .map(|values| values.cloned().collect());
    let output = matches.get_one::<PathBuf>("output");
    let dice = *matches.get_one::<u8>("dice").unwrap();
    let number = *matches.get_one::<u8>("number").unwrap();
    let min = *matches.get_one::<u8>("min").unwrap();
    let max = *matches.get_one::<u8>("max").unwrap();
    let filter_upper = *matches.get_one::<bool>("filter_upper").unwrap();
    let dubeolsik = *matches.get_one::<bool>("dubeolsik").unwrap();

    let cites = include_str!("../../source/hangul/cities");
    let colors = include_str!("../../source/hangul/colors");
    let countries = include_str!("../../source/hangul/countries");
    let daytime = include_str!("../../source/hangul/daytime");
    let elements = include_str!("../../source/hangul/elements");
    let mountains = include_str!("../../source/hangul/mountains");
    let numbers = include_str!("../../source/hangul/numbers");
    let rivers = include_str!("../../source/hangul/rivers");
    let spices = include_str!("../../source/hangul/spices");
    let treasures = include_str!("../../source/hangul/treasures");

    let words = match input {
        Some(v) => v
            .iter()
            .map(|x| read_file(x, min, max, dubeolsik, filter_upper))
            .collect(),
        None => Vec::new(),
    };

    let builtdin_words = match builtin {
        Some(v) => {
            if v.contains(&BuiltinFile::All) {
                vec![
                    read_string(cites, min, max, dubeolsik, filter_upper),
                    read_string(colors, min, max, dubeolsik, filter_upper),
                    read_string(countries, min, max, dubeolsik, filter_upper),
                    read_string(daytime, min, max, dubeolsik, filter_upper),
                    read_string(elements, min, max, dubeolsik, filter_upper),
                    read_string(mountains, min, max, dubeolsik, filter_upper),
                    read_string(numbers, min, max, dubeolsik, filter_upper),
                    read_string(rivers, min, max, dubeolsik, filter_upper),
                    read_string(spices, min, max, dubeolsik, filter_upper),
                    read_string(treasures, min, max, dubeolsik, filter_upper),
                ]
                .concat()
            } else {
                v.iter().fold(Vec::new(), |acc, f| {
                    let words = match f {
                        BuiltinFile::All => Vec::new(),
                        BuiltinFile::Cities => {
                            read_string(cites, min, max, dubeolsik, filter_upper)
                        }
                        BuiltinFile::Colors => {
                            read_string(colors, min, max, dubeolsik, filter_upper)
                        }
                        BuiltinFile::Countries => {
                            read_string(countries, min, max, dubeolsik, filter_upper)
                        }
                        BuiltinFile::DayTime => {
                            read_string(daytime, min, max, dubeolsik, filter_upper)
                        }
                        BuiltinFile::Elements => {
                            read_string(elements, min, max, dubeolsik, filter_upper)
                        }
                        BuiltinFile::Mountains => {
                            read_string(mountains, min, max, dubeolsik, filter_upper)
                        }
                        BuiltinFile::Numbers => {
                            read_string(numbers, min, max, dubeolsik, filter_upper)
                        }
                        BuiltinFile::Rivers => {
                            read_string(rivers, min, max, dubeolsik, filter_upper)
                        }
                        BuiltinFile::Spices => {
                            read_string(spices, min, max, dubeolsik, filter_upper)
                        }
                        BuiltinFile::Treasures => {
                            read_string(treasures, min, max, dubeolsik, filter_upper)
                        }
                    };

                    vec![acc, words].concat()
                })
            }
        }
        None => Vec::new(),
    };

    let mut wordlist = vec![words.concat(), builtdin_words].concat();
    let mut rng = StdRng::from_os_rng();
    wordlist.shuffle(&mut rng);
    let mut wordlist = uniq_words(wordlist);
    wordlist.truncate((dice as usize).pow(number.into()));
    wordlist.sort();
    let index = gen_dice_number(Vec::new(), dice, number.into());

    match output {
        Some(p) => {
            if let Ok(mut file) = File::options().append(true).create(true).open(p) {
                for (w, i) in wordlist.iter().zip(index.iter()) {
                    if dubeolsik {
                        write!(file, "{} {} {}\n", i, dubeolsik_word(w), w).unwrap();
                    } else {
                        write!(file, "{} {}\n", i, w).unwrap();
                    }
                }
            } else {
                eprint!("can not open {}", p.to_str().unwrap());
                exit(1)
            }
        }
        None => {
            for (w, i) in wordlist.iter().zip(index.iter()) {
                if dubeolsik {
                    println!("{} {} {}", i, w, dubeolsik_word(w));
                } else {
                    println!("{} {}", i, w);
                }
            }
        }
    }
}
