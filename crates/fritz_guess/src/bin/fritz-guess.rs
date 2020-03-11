//! TODO

use fritz_guess::{GuessSeparator, MostFrequentLineByLine};
use rayon::prelude::*;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

enum Guess {
    Success(char, PathBuf),
    Fail(PathBuf),
}

impl Display for Guess {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Guess::Success(guess, path) => f.write_str(&format!(
                "guessed '{}' for file: {:?}",
                guess,
                path.display()
            )),
            Guess::Fail(path) => {
                f.write_str(&format!("Guessing failed for file: {:?}", path.display()))
            }
        }
    }
}

/// Basic implementation which supports
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sep = &[',', ';'];

    println!(
        "> fritz-guess: Attempts to guess which separator is used in a CSV (One of the following: {:?}).",
        sep
    );
    println!("> usage: fritz-guess f [<FILE 0>, <FILE 1>, ... <FILE n>] or fritz-guess d <DIR>\n");
    println!(
        "> modes: files by paths, named 'f', example: 'fritz-guess f yellow.csv white.csv'"
    );
    println!("> modes: files in a directory, named 'd', example: 'fritz-guess d .'\n");
    println!("> source: https://github.com/foresterre/fritz\n");

    let mut args = std::env::args().skip(1);
    let first = args.next();

    let paths: Vec<PathBuf> = match first {
        Some(mode) if &mode.to_ascii_lowercase() == "f" => {
            args.map(|arg| PathBuf::from(&arg)).collect()
        }
        Some(mode) if &mode.to_ascii_lowercase() == "d" => {
            if let Some(dir) = args.next() {
                let paths =
                    std::fs::read_dir(&dir).expect(&format!("Unable to read directory: {}", &dir));

                paths
                    .filter_map(|path| {
                        let entry = path.expect("Unable to read dir entry");
                        if entry.path().is_file() {
                            Some(entry.path())
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                println!("No directory found.");
                std::process::exit(-2);
            }
        }
        _ => {
            println!("Unknown mode.");
            std::process::exit(-1);
        }
    };

    let size = paths.len();
    let counter = AtomicUsize::new(0);

    let guesses = paths
        .into_par_iter()
        .map(|path: PathBuf| {
            let mut file = BufReader::new(
                File::open(&path).expect(&format!("Unable to read file '{:?}'.", &path.display())),
            );

            let guesser = MostFrequentLineByLine::try_new(&mut file).expect(&format!(
                "Unable to read file contents for file: {:?}",
                &path.display()
            ));

            if let Ok(guess) = guesser.guess(sep.iter()) {
                let prev = counter.fetch_add(1, Ordering::SeqCst);
                print!("\rcompleted: {}/{}", prev + 1, size);
                Guess::Success(guess, path.to_path_buf())
            } else {
                Guess::Fail(path.to_path_buf())
            }
        })
        .collect::<Vec<Guess>>();

    println!();

    for guess in guesses {
        println!("{}", guess);
    }

    println!("done!, processed {} files", size);

    Ok(())
}
