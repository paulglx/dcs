use clap::Parser;
use dicom::object::open_file;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::path::PathBuf;
use walkdir::WalkDir;

/// DICOM Search CLI Tool - Search and group DICOM files by series
#[derive(Parser, Debug)]
#[command(name = "dcs", version, about)]
struct Cli {
    /// Directory to scan for DICOM files
    directory: PathBuf,
}

/// Information extracted from a DICOM file
#[derive(Debug, Clone)]
struct DicomInfo {
    patient_id: String,
    series_description: String,
    study_description: String,
    file_path: PathBuf,
}

/// Scan a directory recursively for DICOM files and extract their metadata
fn scan_directory(dir: &PathBuf) -> Vec<DicomInfo> {
    // First pass: count total files
    let total_files = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .count() as u64;

    // Create progress bar
    let pb = ProgressBar::new(total_files);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .expect("Invalid progress bar template")
            .progress_chars("█░"),
    );

    let mut results = Vec::new();

    // Second pass: process files with progress bar
    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path().to_path_buf();
        
        // Update progress bar with current filename
        if let Some(filename) = path.file_name() {
            pb.set_message(filename.to_string_lossy().to_string());
        }

        if let Some(info) = extract_tags(path) {
            results.push(info);
        }

        pb.inc(1);
    }

    pb.finish_and_clear();

    results
}

/// Extract DICOM tags from a file
fn extract_tags(file_path: PathBuf) -> Option<DicomInfo> {
    let obj = open_file(&file_path).ok()?;

    // PatientID (0010,0020)
    let patient_id = obj
        .element_by_name("PatientID")
        .ok()
        .and_then(|e| e.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    // SeriesDescription (0008,103E)
    let series_description = obj
        .element_by_name("SeriesDescription")
        .ok()
        .and_then(|e| e.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    // StudyDescription (0008,1030)
    let study_description = obj
        .element_by_name("StudyDescription")
        .ok()
        .and_then(|e| e.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    Some(DicomInfo {
        patient_id,
        series_description,
        study_description,
        file_path,
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

        // Get patient and study info from the first file
        let first = &files[0];
        let patient_id = &first.patient_id;
        let study_description = &first.study_description;

        println!("Series: {}", series_desc);
        println!("  Patient ID: {}", patient_id);
        println!("  Study: {}", study_description);
        println!("  Files: {} (first: {})", files.len(), first.file_path.display());
        println!();
    }
}

fn main() {
    let cli = Cli::parse();

    if !cli.directory.exists() {
        eprintln!("Error: Directory '{}' does not exist", cli.directory.display());
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
    print_results(groups);
}
