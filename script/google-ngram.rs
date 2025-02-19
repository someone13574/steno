#!/usr/bin/env cargo-eval
//! ```cargo
//! [dependencies]
//! clap = { version = "4.5.30", features = ["derive"] }
//! tokio = { version = "1.43.0", features = ["full"] }
//! futures = { version = "0.3.31" }
//! reqwest = { version = "0.12.12", features = ["stream"] }
//! flate2 = { version = "1.0.35" }
//! ```

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use clap::Parser;
use flate2::read::GzDecoder;
use futures::StreamExt;
use tokio::io::AsyncWriteExt;

#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    /// Dataset to download (eg. 'googlebooks-eng-all')
    dataset: String,

    /// Version to download (eg. '20120701')
    version: String,

    /// Dictionary size
    count: usize,

    /// Clear temp directory
    #[arg(short, long)]
    clear: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let target_dir = Path::new(".").canonicalize().unwrap().join("temp");

    if args.clear {
        println!("Clearing temp directory...");
        fs::remove_dir_all(&target_dir).unwrap();
    }

    fs::create_dir_all(&target_dir).unwrap();

    // Download 1gram data
    let mut downloads = Vec::new();
    let mut paths = Vec::new();

    for char in 'a'..='z' {
        let filename = format!("{}-1gram-{}-{}.gz", args.dataset, args.version, char);
        let path = target_dir.join(&filename).to_path_buf();
        paths.push(path.clone());

        if !path.exists() {
            downloads.push(async move {
                download_file(
                    format!("http://storage.googleapis.com/books/ngrams/books/{filename}"),
                    path,
                )
                .await
            });
        }
    }

    for result in futures::future::join_all(downloads).await {
        result.unwrap();
    }

    // Filter and combine counts
    let mut counts = paths
        .iter()
        .flat_map(|path| {
            let counts = process_file(&path);
            counts.into_iter().take(args.count).collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    counts.sort_by(|a, b| b.1.cmp(&a.1));
    counts = counts[..args.count.min(counts.len())].to_vec();

    // Write to file
    let mut file = fs::File::create(target_dir.join("word_list.txt")).unwrap();
    for (word, _) in counts {
        writeln!(file, "{word}").unwrap();
    }
}

async fn download_file(url: String, path: PathBuf) -> Result<(), String> {
    println!("Downloading {url} to {path:?}");

    let mut file = tokio::fs::File::create(path).await.unwrap();
    let mut stream = reqwest::get(url.clone()).await.unwrap().bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap();
        file.write_all(&chunk).await.unwrap();
    }

    file.flush().await.unwrap();

    println!("Finished {url}");

    Ok(())
}

fn process_file(path: &PathBuf) -> Vec<(String, u64)> {
    println!("Processing {path:?}");

    let file = fs::File::open(path).unwrap();
    let decoder = GzDecoder::new(file);
    let reader = BufReader::new(decoder);

    let mut counts = HashMap::new();
    let mut current_1gram = String::new();
    let mut current_matches: u64 = 0;
    let mut current_vols: u64 = 0;

    for line in reader.lines() {
        let line = line.unwrap();
        let parts = line.split('\t').collect::<Vec<_>>();

        let this_year = parts[1].parse::<u64>().unwrap();
        let this_matches = parts[2].parse::<u64>().unwrap();
        let this_vols = parts[3].parse::<u64>().unwrap();

        // Process 1gram
        let this_1gram = parts[0]
            .split('_')
            .next()
            .unwrap()
            .split(".")
            .next()
            .unwrap()
            .trim()
            .to_string();

        // Filter
        if !this_1gram
            .chars()
            .all(|c| c.is_ascii_alphabetic() && c.is_lowercase())
        {
            continue;
        }

        if this_1gram.len() <= 2 {
            continue;
        }

        if this_year < 1980 {
            continue;
        }

        // Update
        if this_1gram == current_1gram {
            current_matches += this_matches;
            current_vols += this_vols;
        } else {
            // Update hashmap
            counts
                .entry(current_1gram.clone())
                .and_modify(|(matches, vols)| {
                    *matches += current_matches;
                    *vols += current_vols;
                })
                .or_insert((current_matches, current_vols));

            // Update variables
            current_1gram = this_1gram;
            current_matches = this_matches;
            current_vols = this_vols;
        }
    }

    // Filter and sort
    let mut counts = counts
        .into_iter()
        .filter(|(_, (_, vols))| *vols >= 1000)
        .map(|(word, (matches, _))| (word, matches))
        .collect::<Vec<_>>();
    counts.sort_by(|a, b| b.1.cmp(&a.1));
    counts
}
