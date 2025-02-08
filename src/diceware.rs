use rand::{rngs::StdRng, Rng, SeedableRng};

pub fn select_words(words: &[String], count: u8, delimiter: &str) -> String {
    let mut rng = StdRng::from_os_rng();
    let len = words.len();
    let words: Vec<String> = (0..count)
        .map(|_| {
            let index = rng.random_range(0..len);
            words[index].clone()
        })
        .collect();
    words.join(delimiter)
}
