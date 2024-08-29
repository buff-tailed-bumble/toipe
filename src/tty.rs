use std::fs::File;
use std::io::{Read, Result, Stdin};

use crate::config::ToipeConfig;

pub enum Tty {
    Stdin(Stdin),
    File(File),
}

impl From<File> for Tty {
    fn from(value: File) -> Self {
        Tty::File(value)
    }
}

impl From<Stdin> for Tty {
    fn from(value: Stdin) -> Self {
        Tty::Stdin(value)
    }
}

impl Tty {
    pub fn new(config: &ToipeConfig) -> Result<Self> {
        if config.is_stdin_tty {
            Ok(std::io::stdin().into())
        } else {
            Ok(termion::get_tty()?.into())
        }
    }

    pub fn map<T>(&mut self, mut f: impl FnMut(&mut dyn Read) -> T) -> T {
        match self {
            Self::Stdin(stdin) => f(&mut stdin.lock()),
            Self::File(file) => f(file),
        }
    }

    pub fn is_stdin(&self) -> bool {
        matches!(self, Self::Stdin(_))
    }
}
