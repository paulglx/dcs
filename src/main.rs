mod parse;
mod scan;
mod search;

use clap::Parser;
use colored::Colorize;
use dicom::core::Tag;
use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::Arc;

use parse::{group_by_series, DicomInfo};
use scan::scan_directory;
use search::fuzzy_filter_series;

#[derive(Parser, Debug)]
#[command(name = "dcs", version, about)]
struct Cli {
    directory: PathBuf,
    search: Option<String>,
    #[arg(long)]
    tag: Option<String>,
}

fn parse_tag_arg(s: &str) -> Tag {
    if s.len() != 8 {
        eprintln!("Error: tag must be exactly 8 hex characters (e.g. 00200037)");
        std::process::exit(1);
    }
    let group = u16::from_str_radix(&s[..4], 16).unwrap_or_else(|_| {
        eprintln!("Error: invalid hex in tag group: {}", &s[..4]);
        std::process::exit(1);
    });
    let element = u16::from_str_radix(&s[4..], 16).unwrap_or_else(|_| {
        eprintln!("Error: invalid hex in tag element: {}", &s[4..]);
        std::process::exit(1);
    });
    Tag(group, element)
}

fn print_tag_values(files: &[DicomInfo], tag: Tag) {
    let values: BTreeSet<&str> = files
        .iter()
        .filter_map(|f| f.extra_tag_value.as_deref())
        .collect();
    if !values.is_empty() {
        let count = values.len();
        let joined = values
            .into_iter()
            .map(|v| v.truecolor(255, 165, 0).to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let suffix = if count >= 3 {
            format!(" {}", format!("({} distinct)", count).dimmed())
        } else {
            String::new()
        };
        println!(
            "  Tag ({:04X},{:04X}): {}{}",
            tag.group(),
            tag.element(),
            joined,
            suffix
        );
    }
}

fn print_results(groups: HashMap<Arc<str>, Vec<DicomInfo>>, extra_tag: Option<Tag>) {
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
        if let Some(tag) = extra_tag {
            print_tag_values(&files, tag);
        }
        println!();
    }
}

fn print_filtered_results(matches: Vec<(Arc<str>, Vec<DicomInfo>, u32)>, extra_tag: Option<Tag>) {
    if matches.is_empty() {
        println!("No matching series found.");
        return;
    }

    for (series_desc, mut files, _) in matches {
        files.sort_by(|a, b| a.file_path.cmp(&b.file_path));

        let first = &files[0];

        println!("Series: {}", series_desc.bold().blue());
        println!(
            "  Files: {} (first: {})",
            files.len(),
            first.file_path.display()
        );
        if let Some(tag) = extra_tag {
            print_tag_values(&files, tag);
        }
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

    let extra_tag = cli.tag.as_deref().map(parse_tag_arg);

    println!("Scanning directory: {}", cli.directory.display());
    println!();

    let files = scan_directory(&cli.directory, extra_tag);
    let groups = group_by_series(files);

    if let Some(ref pattern) = cli.search {
        let matches = fuzzy_filter_series(groups, pattern);
        print_filtered_results(matches, extra_tag);
    } else {
        print_results(groups, extra_tag);
    }
}
