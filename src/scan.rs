use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use jwalk::WalkDir;
use rayon::prelude::*;
use std::path::PathBuf;

use crate::parse::{extract_tags, has_dicom_preamble, DicomInfo};

pub fn scan_directory(dir: &PathBuf) -> Vec<DicomInfo> {
    let file_paths: Vec<PathBuf> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path())
        .filter(|p| has_dicom_preamble(p))
        .collect();

    let pb = ProgressBar::new(file_paths.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}] {pos}/{len}")
            .expect("Invalid progress bar template")
            .progress_chars("█░"),
    );

    let results: Vec<DicomInfo> = file_paths
        .par_iter()
        .progress_with(pb)
        .filter_map(|path| extract_tags(path))
        .collect();

    results
}
