//! Utilities for generating/selecting new (random) words for the typing
//! test.

use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Cursor, Seek};
use std::path::PathBuf;

use rand::seq::SliceRandom;
use rand::Rng;

use rand::prelude::ThreadRng;

use crate::trie::Trie;

pub struct RawWordSelector<T: Seek + io::Read> {
    trie: Trie,
    phantom: std::marker::PhantomData<T>,
}

impl<T: Seek + io::Read> RawWordSelector<T> {
    pub fn new(reader: BufReader<T>) -> Result<Self, io::Error> {
        let mut trie = Trie::new();
        for line in reader.lines() {
            match line {
                Ok(string) => {
                    if let Err(err) = trie.insert(&string.to_ascii_lowercase()) {
                        return Err(err.into());
                    }
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }

        trie.compress()
            .map(|t| Self {
                trie: t,
                phantom: std::marker::PhantomData,
            })
            .map_err(|e| e.into())
    }

    fn new_word_raw(&mut self, rng: &mut ThreadRng) -> Result<String, io::Error> {
        self.trie
            .sample(rng.gen_range(0..self.trie.num_words()))
            .map_err(|e| e.into())
    }
}

impl RawWordSelector<File> {
    /// Create from a file at a path given by a [`PathBuf`].
    ///
    /// Please ensure that assumptions defined at
    /// [`RawWordSelector#assumptions`] are valid for this file.
    pub fn from_path(word_list_path: PathBuf) -> Result<Self, io::Error> {
        let file = File::open(word_list_path)?;

        let reader = BufReader::new(file);

        Self::new(reader)
    }
}

impl RawWordSelector<Cursor<String>> {
    /// Create from a String representing the word list file.
    ///
    /// Please ensure that assumptions defined at
    /// [`RawWordSelector#assumptions`] are valid for the contents.
    pub fn from_string(word_list: String) -> Result<Self, io::Error> {
        let cursor = Cursor::new(word_list);
        let reader = BufReader::new(cursor);

        RawWordSelector::new(reader)
    }
}

/// Describes a thing that provides new words.
pub trait WordSelector {
    /// Returns a new word.
    fn new_word(&mut self) -> Result<String, io::Error>;

    /// Returns a [`Vec`] containing `num_words` words.
    fn new_words(&mut self, num_words: usize) -> Result<Vec<String>, io::Error> {
        (0..num_words).map(|_| self.new_word()).collect()
    }
}

impl<T: Seek + io::Read> WordSelector for RawWordSelector<T> {
    fn new_word(&mut self) -> Result<String, io::Error> {
        let mut rng = rand::thread_rng();
        Ok(self.new_word_raw(&mut rng)?.to_ascii_lowercase())
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
