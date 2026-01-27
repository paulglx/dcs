mod parse;
mod scan;
mod search;

use clap::Parser;
use colored::Colorize;
use std::collections::HashMap;
use std::path::PathBuf;

use parse::{group_by_series, DicomInfo};
use scan::scan_directory;
use search::fuzzy_filter_series;

#[derive(Parser, Debug)]
#[command(name = "dcs", version, about)]
struct Cli {
    directory: PathBuf,
    search: Option<String>,
}

fn print_results(groups: HashMap<String, Vec<DicomInfo>>) {
    if groups.is_empty() {
        println!("No DICOM files found.");
        return;
    }

    let mut series_list: Vec<_> = groups.into_iter().collect();
    series_list.sort_by(|a, b| a.0.cmp(&b.0));

    for (series_desc, mut files) in series_list {
        files.sort_by(|a, b| a.file_path.cmp(&b.file_path));

        let first = &files[0];

        println!("Series: {}", series_desc.bold().blue());
        println!(
            "  Files: {} (first: {})",
            files.len(),
            first.file_path.display()
        );
        println!();
    }
}

fn print_filtered_results(matches: Vec<(String, Vec<DicomInfo>, u32)>) {
    if matches.is_empty() {
        println!("No matching series found.");
        return;
    }

    for (series_desc, mut files, _) in matches {
        // Sort files by path first file
        files.sort_by(|a, b| a.file_path.cmp(&b.file_path));

        let first = &files[0];

        println!("Series: {}", series_desc.bold().blue());
        println!(
            "  Files: {} (first: {})",
            files.len(),
            first.file_path.display()
        );
        println!();
    }
}

fn main() {
    let cli = Cli::parse();

    if !cli.directory.exists() {
        eprintln!(
            "Error: Directory '{}' does not exist",
            cli.directory.display()
        );
        std::process::exit(1);
    }

    if !cli.directory.is_dir() {
        eprintln!("Error: '{}' is not a directory", cli.directory.display());
        std::process::exit(1);
    }

    println!("Scanning directory: {}", cli.directory.display());
    println!();

    let files = scan_directory(&cli.directory);
    let groups = group_by_series(files);

    if let Some(ref pattern) = cli.search {
        let matches = fuzzy_filter_series(groups, pattern);
        print_filtered_results(matches);
    } else {
        print_results(groups);
    }
}
