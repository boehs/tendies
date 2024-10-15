//! Comments are aquired from the-eye.eu: https://the-eye.eu/redarcs/
//! The files are .zst compressed, so we need to decompress them first.
//! The files are very large, so it is better to stream them.
//! Pipeline is zst stream → serde_json stream → ??
#![feature(lazy_cell)]

use std::fs::File;
use std::io::BufReader;

use serde::Deserialize;

use core::cell::LazyCell;

#[derive(Deserialize, Debug)]
struct Comment {
    id: String,
    parent_id: Option<String>,
    link_id: String,
    subreddit: String,
    body: String,
}

// this is not exhaustive, the intent is to eliminate things that under no circumstances could be a ticker
const IGNORED_TICKERS: [&str; 12] = [
    "DD", "WSB", "HODL", "US", "CEO", "GDP", "I", "FDA", "SEC", "PR", "IIRC", "LOL",
];

impl Comment {
    fn get_tickers(&self) -> Vec<&str> {
        let ticker_re = LazyCell::new(|| regex::Regex::new(r"\b[A-Z]{1,5}\b").unwrap());
        // if the comment is all caps, disregard it
        if self.body.to_ascii_uppercase() == self.body {
            return vec![];
        }
        ticker_re
            .find_iter(&self.body)
            .map(|m| m.as_str())
            .filter(|ticker| !IGNORED_TICKERS.contains(ticker))
            .collect()
    }

    fn is_deleted(&self) -> bool {
        self.body == "[deleted]"
    }
}

fn main() {
    // Open the file
    let Ok(file) = File::open("data/comments.zst") else {
        eprintln!("Could not open file");
        return;
    };
    let reader = BufReader::new(file);

    let decoder = zstd::stream::Decoder::new(reader).unwrap();

    let stream = serde_json::Deserializer::from_reader(decoder).into_iter::<Comment>();

    for comment in stream {
        let comment = comment.unwrap();
        if comment.is_deleted() {
            continue;
        }
        println!("{:?}", comment);
        println!("{:?}", comment.get_tickers());
    }
}
