//! Utilities for generating/selecting new (random) words for the typing
//! test.

use std::collections::VecDeque;
use std::io;

use rand::seq::SliceRandom;
use rand::Rng;

use rand::prelude::ThreadRng;

use crate::trie::Trie;

pub struct RawWordSelector {
    trie: Trie,
}

impl RawWordSelector {
    pub fn from_iter<T: Iterator<Item = Result<String, io::Error>>>(
        iter: T,
    ) -> Result<Self, io::Error> {
        let mut trie = Trie::new();
        for elem in iter {
            match elem {
                Ok(word) => {
                    if let Err(err) = trie.insert(&word) {
                        return Err(err.into());
                    }
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }

        trie.compress()
            .map(|t| Self { trie: t })
            .map_err(|e| e.into())
    }

    fn new_word_raw(&mut self, rng: &mut ThreadRng) -> Result<String, io::Error> {
        self.trie
            .sample(rng.gen_range(0..self.trie.num_words()))
            .map_err(|e| e.into())
    }
}

/// Describes a thing that provides new words.
pub trait WordSelector {
    /// Returns a new word.
    fn new_word(&mut self) -> Result<String, io::Error>;

    /// Returns a [`Vec`] containing `num_words` words.
    fn new_words(&mut self, num_words: usize) -> Result<Vec<String>, io::Error> {
        let mut words = Vec::<String>::new();
        for _ in 0..num_words {
            let word = self.new_word()?;
            for part in word.split_whitespace() {
                words.push(part.to_string());
            }
        }
        Ok(words)
    }
}

impl WordSelector for RawWordSelector {
    fn new_word(&mut self) -> Result<String, io::Error> {
        let mut rng = rand::thread_rng();
        Ok(self.new_word_raw(&mut rng)?)
    }
}

pub struct NumberGeneratingWordSelector {
    selector: Box<dyn WordSelector>,
    number_chance: f64,
    number_max: u64,
}

impl NumberGeneratingWordSelector {
    pub fn from_word_selector(
        word_selector: Box<dyn WordSelector>,
        number_chance: f64,
        number_max: u64,
    ) -> Self {
        Self {
            selector: word_selector,
            number_chance,
            number_max,
        }
    }
}

impl WordSelector for NumberGeneratingWordSelector {
    fn new_word(&mut self) -> Result<String, io::Error> {
        let mut rng = rand::thread_rng();
        if !rng.gen_bool(self.number_chance) {
            return self.selector.new_word();
        }
        let num = rng.gen_range(0..self.number_max);
        Ok(num.to_string())
    }
}

/// Wraps another word selector, taking words from it and adding punctuation to the end of or
/// around words with a configurable chance. Will capitalize the next word when an end-of-sentence
/// punctuation mark is used.
pub struct PunctuatedWordSelector {
    selector: Box<dyn WordSelector>,
    next_is_capital: bool,
    punctuation_chance: f64,
}

enum PunctuationType {
    Capitaizing(char),
    Ending(char),
    Starting(char),
    Surrounding(char, char),
}

const PUNCTUATION: [PunctuationType; 33] = [
    PunctuationType::Capitaizing('!'),
    PunctuationType::Capitaizing('?'),
    PunctuationType::Capitaizing('.'),
    PunctuationType::Ending(','),
    PunctuationType::Ending(':'),
    PunctuationType::Ending(';'),
    PunctuationType::Starting(':'),
    PunctuationType::Starting('@'),
    PunctuationType::Starting('#'),
    PunctuationType::Starting('$'),
    PunctuationType::Starting('%'),
    PunctuationType::Starting('^'),
    PunctuationType::Starting('&'),
    PunctuationType::Starting('*'),
    PunctuationType::Starting('~'),
    PunctuationType::Starting('/'),
    PunctuationType::Starting('\\'),
    PunctuationType::Starting('_'),
    PunctuationType::Starting('-'),
    PunctuationType::Starting('='),
    PunctuationType::Starting('+'),
    PunctuationType::Surrounding('\'', '\''),
    PunctuationType::Surrounding('"', '"'),
    PunctuationType::Surrounding('(', ')'),
    PunctuationType::Surrounding('{', '}'),
    PunctuationType::Surrounding('<', '>'),
    PunctuationType::Surrounding('[', ']'),
    PunctuationType::Surrounding('%', '%'),
    PunctuationType::Surrounding('^', '$'),
    PunctuationType::Surrounding('*', '*'),
    PunctuationType::Surrounding('`', '`'),
    PunctuationType::Surrounding('/', '/'),
    PunctuationType::Surrounding('|', '|'),
];

impl PunctuatedWordSelector {
    /// Creates a PunctuatedWordSelector from another WordSelector, allowing the selection of the
    /// chance of punctuation.
    pub fn from_word_selector(
        word_selector: Box<dyn WordSelector>,
        punctuation_chance: f64,
    ) -> Self {
        Self {
            selector: word_selector,
            next_is_capital: true,
            punctuation_chance,
        }
    }
}

impl WordSelector for PunctuatedWordSelector {
    fn new_word(&mut self) -> Result<String, io::Error> {
        let mut rng = rand::thread_rng();

        let mut word = self.selector.new_word()?;

        let will_punctuate = rng.gen_bool(self.punctuation_chance);
        if will_punctuate || self.next_is_capital {
            let mut chars: VecDeque<char> = word.chars().collect();
            if self.next_is_capital {
                // some unicode chars map to multiple chars when uppercased.
                for c in chars
                    .pop_front()
                    .expect("got empty word")
                    .to_uppercase()
                    .rev()
                {
                    chars.push_front(c)
                }
                self.next_is_capital = false;
            }
            if will_punctuate {
                match PUNCTUATION
                    .choose(&mut rng)
                    .expect("only returns none if the slice is empty")
                {
                    PunctuationType::Capitaizing(c) => {
                        self.next_is_capital = true;
                        chars.push_back(*c)
                    }
                    PunctuationType::Ending(c) => chars.push_back(*c),
                    PunctuationType::Starting(c) => chars.push_front(*c),
                    PunctuationType::Surrounding(opening, closing) => {
                        chars.push_front(*opening);
                        chars.push_back(*closing);
                    }
                }
            }
            word = chars.into_iter().collect();
        }
        Ok(word)
    }
}
