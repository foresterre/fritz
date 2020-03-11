//! Strategies for guessing which separator occurs most often in a CSV file.
//! Originally made to guess whether a CSV file uses the `;` or `,` character as separator.
//!
//! Part of the [Fritz](https://github.com/foresterre/fritz) workbook toolset.

use std::collections::{BTreeMap, HashMap};
use std::io::Read;
use thiserror::Error;

#[cfg(test)]
#[macro_use]
extern crate parameterized;

/// Decent effort guesser which implements guessing using a line-by-line winner takes all strategy.
/// For a line, the winner is the separator which occurs most frequent.
/// The overall winner is the separator which takes most wins.
pub struct MostFrequentLineByLine {
    /// Contents of a file or
    content: String,
}

impl MostFrequentLineByLine {
    pub fn try_new<R: Read>(source: &mut R) -> Result<MostFrequentLineByLine, Error> {
        let mut buffer = String::new();

        source
            .read_to_string(&mut buffer)
            .map_err(|err| Error::Io(err))?;

        Ok(MostFrequentLineByLine {
            content: buffer.to_owned(),
        })
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error ({0})")]
    Io(std::io::Error),

    #[error("The file doesn't contain any of the expected delimiters.")]
    NoDelimiterFound,
}

trait IsOneOf<'v, V: 'v> {
    fn is_one_of<I: Iterator<Item = &'v V>>(&self, iter: I) -> bool;
}

impl<'v, V: PartialEq + 'v> IsOneOf<'v, V> for V {
    fn is_one_of<I: Iterator<Item = &'v V>>(&self, iter: I) -> bool {
        iter.filter(|f| *f == self).next().is_some()
    }
}

pub trait GuessSeparator {
    fn guess<'a>(&self, separators: impl Iterator<Item = &'a char> + Clone) -> Result<char, Error>;
}

impl GuessSeparator for MostFrequentLineByLine {
    /// Check which of the provided separators occurs most often per line.
    /// That character which occurs most often 'wins' for a line.
    /// The character which wins most lines will be accepted as guess.
    fn guess<'a>(&self, separators: impl Iterator<Item = &'a char> + Clone) -> Result<char, Error> {
        self.content
            .lines()
            .flat_map(|line| {
                line.chars()
                    .filter(|item| item.is_one_of(separators.clone()))
                    .fold(HashMap::<char, usize>::new(), |mut acc, char| {
                        *acc.entry(char).or_default() += 1;
                        acc
                    })
                    .iter()
                    .max_by(|lhs, rhs| lhs.1.cmp(&rhs.1))
                    .map(|(char, _)| *char)
            })
            // folds using a BTreeMap instead of a HashMap to ensure consistent outputs
            .fold(BTreeMap::<char, usize>::new(), |mut acc, char| {
                *acc.entry(char).or_default() += 1;
                acc
            })
            .iter()
            .max_by(|lhs, rhs| lhs.1.cmp(&rhs.1))
            .ok_or_else(|| Error::NoDelimiterFound)
            .map(|(char, _)| *char)
    }
}

#[cfg(test)]
mod tests {
    use crate::{GuessSeparator, MostFrequentLineByLine};

    // FIXME: test properly <3

    ide!();

    #[parameterized(
        text = {
            "a;a",                              // single line
            "a;a\na;a",                         // multi line
            "a;b;c,c",
            "a;b;c\na;a,b,c\na;b;c\na;b;c\n",
            "a,b,c;d\na,b,c;d\na,b,c;d",
        },
        expected = {
            ';',
            ';',
            ';',
            ';',
            ',',
        }
    )]
    fn positive_well_formed_guesses(text: &str, expected: char) {
        let mut source = text.as_bytes();

        let guesser = MostFrequentLineByLine::try_new(&mut source).unwrap();
        let guess = guesser.guess([';', ','].iter()).unwrap();

        assert_eq!(guess, expected);
    }

    #[parameterized(
        text = {
            "a,a\na;a",
            "a,a\na;a",
            "a;a\na,a",
            "a;a\na,a",
        },
        seps = {
            &[';', ','],
            &[',', ';'],
            &[';', ','],
            &[',', ';'],
        },
        expected = {
            ';',
            ';',
            ';',
            ';',
        }
    )]
    fn equal_counts_have_char_ordering(text: &str, seps: &[char], expected: char) {
        let mut source = text.as_bytes();

        let guesser = MostFrequentLineByLine::try_new(&mut source).unwrap();
        let guess = guesser.guess(seps.iter()).unwrap();

        assert_eq!(guess, expected);
    }
}
