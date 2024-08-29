use std::{
    fs::File,
    io::{BufRead, BufReader, Cursor, Error, ErrorKind, Read},
    path::PathBuf,
};

use crate::{
    config::ToipeConfig,
    wordlists::{BuiltInWordlist, OS_WORDLIST_PATH},
};

pub struct WordStream {
    stream: Box<dyn Read>,
    preserve_whitespace: bool,
}

impl WordStream {
    pub fn new(config: &ToipeConfig) -> Result<Self, Error> {
        let stdin = std::io::stdin().lock();

        let stream: Box<dyn Read> = if !termion::is_tty(&stdin) {
            Box::new(stdin)
        } else if let Some(path) = &config.wordlist_file {
            Box::new(File::open(PathBuf::from(path))?)
        } else if let Some(contents) = config.wordlist.contents().map(|c| c.to_string()) {
            Box::new(Cursor::<String>::new(contents))
        } else if let BuiltInWordlist::OS = config.wordlist {
            Box::new(File::open(PathBuf::from(OS_WORDLIST_PATH))?)
        } else {
            return Err(Error::new(
                ErrorKind::Other,
                "Could not determine word source",
            ));
        };

        Ok(Self {
            stream,
            preserve_whitespace: config.preserve_whitespace,
        })
    }

    pub fn into_iter(self) -> impl Iterator<Item = Result<String, Error>> {
        let preserve_whitespace = self.preserve_whitespace;
        let reader = BufReader::new(self.stream);
        reader
            .lines()
            .map(move |result| match result {
                Ok(line) => {
                    if preserve_whitespace {
                        vec![Ok(line)].into_iter()
                    } else {
                        line.split_whitespace()
                            .map(|s| Ok(s.to_string()))
                            .collect::<Vec<_>>()
                            .into_iter()
                    }
                }
                Err(err) => vec![Err(err)].into_iter(),
            })
            .flatten()
    }
}
