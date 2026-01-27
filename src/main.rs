use clap::Parser;
use dicom::core::Tag;
use dicom::dictionary_std::tags::PIXEL_DATA;
use dicom::object::OpenFileOptions;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// DICOM Search CLI Tool - Search and group DICOM files by series
#[derive(Parser, Debug)]
#[command(name = "dcs", version, about)]
struct Cli {
    /// Directory to scan for DICOM files
    directory: PathBuf,

    /// Fuzzy search pattern to filter series descriptions
    search: Option<String>,
}

/// Information extracted from a DICOM file
#[derive(Debug, Clone)]
struct DicomInfo {
    series_description: String,
    file_path: PathBuf,
}

/// Scan a directory recursively for DICOM files and extract their metadata
fn scan_directory(dir: &PathBuf) -> Vec<DicomInfo> {
    // Single pass: collect all file paths
    let file_paths: Vec<PathBuf> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .collect();

    // Create progress bar
    let pb = ProgressBar::new(file_paths.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}] {pos}/{len}")
            .expect("Invalid progress bar template")
            .progress_chars("█░"),
    );

    // Process files in parallel with progress tracking
    let results: Vec<DicomInfo> = file_paths
        .par_iter()
        .progress_with(pb)
        .filter_map(|path| extract_tags(path))
        .collect();

    results
}

/// Extract DICOM tags from a file (reads only up to pixel data)
fn extract_tags(file_path: &Path) -> Option<DicomInfo> {
    let obj = OpenFileOptions::new()
        .read_until(PIXEL_DATA)
        .open_file(file_path)
        .ok()?;

    // SeriesDescription (0008,103E)
    const SERIES_DESCRIPTION: Tag = Tag(0x0008, 0x103E);
    let series_description = obj
        .element(SERIES_DESCRIPTION)
        .ok()
        .and_then(|e| e.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    Some(DicomInfo {
        series_description,
        file_path: file_path.to_path_buf(),
    })
}

/// Group DICOM files by series description
fn group_by_series(files: Vec<DicomInfo>) -> HashMap<String, Vec<DicomInfo>> {
    let mut groups: HashMap<String, Vec<DicomInfo>> = HashMap::new();

    for info in files {
        groups
            .entry(info.series_description.clone())
            .or_default()
            .push(info);
    }

    groups
}

/// Filter series by fuzzy matching against the pattern, returning matches sorted by score
fn fuzzy_filter_series(
    groups: HashMap<String, Vec<DicomInfo>>,
    pattern: &str,
) -> Vec<(String, Vec<DicomInfo>, u32)> {
    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(pattern, CaseMatching::Ignore, Normalization::Smart);

    let mut matches: Vec<(String, Vec<DicomInfo>, u32)> = groups
        .into_iter()
        .filter_map(|(description, files)| {
            let mut buf = Vec::new();
            let haystack = nucleo_matcher::Utf32Str::new(&description, &mut buf);
            pattern
                .score(haystack, &mut matcher)
                .map(|score| (description, files, score))
        })
        .collect();

    // Sort by score descending (best matches first)
    matches.sort_by(|a, b| b.2.cmp(&a.2));

    matches
}

/// Print the grouped results
fn print_results(groups: HashMap<String, Vec<DicomInfo>>) {
    if groups.is_empty() {
        println!("No DICOM files found.");
        return;
    }

    let mut series_list: Vec<_> = groups.into_iter().collect();
    series_list.sort_by(|a, b| a.0.cmp(&b.0));

    for (series_desc, mut files) in series_list {
        // Sort files by path to get consistent "first" file
        files.sort_by(|a, b| a.file_path.cmp(&b.file_path));

        let first = &files[0];

        println!("Series: {}", series_desc);
        println!(
            "  Files: {} (first: {})",
            files.len(),
            first.file_path.display()
        );
        println!();
    }
}

/// Print filtered results with match scores
fn print_filtered_results(matches: Vec<(String, Vec<DicomInfo>, u32)>) {
    if matches.is_empty() {
        println!("No matching series found.");
        return;
    }

    for (series_desc, mut files, _) in matches {
        // Sort files by path to get consistent "first" file
        files.sort_by(|a, b| a.file_path.cmp(&b.file_path));

        let first = &files[0];

        println!("Series: {}", series_desc);
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
