use dicom::core::Tag;
use dicom::object::OpenFileOptions;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub const SERIES_DESCRIPTION: Tag = Tag(0x0008, 0x103E);
// Tag after SERIES_DESCRIPTION to ensure it gets read (read_until is exclusive)
const SERIES_DESCRIPTION_NEXT: Tag = Tag(0x0008, 0x103F);

pub fn has_dicom_preamble(path: &Path) -> bool {
    let Ok(mut file) = File::open(path) else {
        return false;
    };
    let mut buf = [0u8; 132];
    if file.read_exact(&mut buf).is_err() {
        return false;
    };
    &buf[128..132] == b"DICM"
}

#[derive(Debug, Clone)]
pub struct DicomInfo {
    pub series_description: String,
    pub file_path: std::path::PathBuf,
}

pub fn extract_tags(file_path: &Path) -> Option<DicomInfo> {
    let obj = OpenFileOptions::new()
        .read_until(SERIES_DESCRIPTION_NEXT)
        .open_file(file_path)
        .ok()?;

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

pub fn group_by_series(files: Vec<DicomInfo>) -> HashMap<String, Vec<DicomInfo>> {
    let mut groups: HashMap<String, Vec<DicomInfo>> = HashMap::new();

    for info in files {
        groups
            .entry(info.series_description.clone())
            .or_default()
            .push(info);
    }

    groups
}
