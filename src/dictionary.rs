use std::io::BufRead;

use gpui::{App, AppContext, Global};
use rand::seq::IteratorRandom;
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "assets/dictionaries"]
#[include = "*"]
struct Dictionaries;

pub struct Dictionary {
    words: Vec<String>,
}

impl Global for Dictionary {}

impl Dictionary {
    pub fn new(id: &str, truncate: usize) -> Self {
        let data = Dictionaries::get(format!("{id}.txt").as_str()).unwrap();
        let words = data
            .data
            .lines()
            .take(truncate)
            .map(|line| line.unwrap().to_string())
            .collect::<Vec<_>>();
        Self { words }
    }

    pub fn set_global(self, cx: &mut App) {
        cx.set_global(self);
    }

    pub fn random_text(word_count: usize, cx: &mut App) -> String {
        let mut rng = rand::rng();
        cx.read_global(|this: &Self, _cx| {
            this.words
                .iter()
                .choose_multiple(&mut rng, word_count)
                .into_iter()
                .map(|word| word.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        })
    }
}
