use dicom::core::Tag;
use dicom::object::OpenFileOptions;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

pub const SERIES_DESCRIPTION: Tag = Tag(0x0008, 0x103E);
// Tag after SERIES_DESCRIPTION to ensure it gets read (read_until is exclusive)
const SERIES_DESCRIPTION_NEXT: Tag = Tag(0x0008, 0x103F);

fn has_dicom_preamble(path: &Path) -> bool {
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
    pub series_description: Arc<str>,
    pub file_path: std::path::PathBuf,
    pub extra_tag_value: Option<String>,
}

pub fn extract_tags(file_path: &Path, extra_tag: Option<Tag>) -> Option<DicomInfo> {
    if !has_dicom_preamble(file_path) {
        return None;
    }

    let read_until = match extra_tag {
        Some(t) => std::cmp::max(SERIES_DESCRIPTION_NEXT, Tag(t.group(), t.element() + 1)),
        None => SERIES_DESCRIPTION_NEXT,
    };

    let obj = OpenFileOptions::new()
        .read_until(read_until)
        .open_file(file_path)
        .ok()?;

    let series_description: Arc<str> = obj
        .element(SERIES_DESCRIPTION)
        .ok()
        .and_then(|e| e.to_str().ok())
        .map(|s| Arc::from(s.as_ref()))
        .unwrap_or_else(|| Arc::from("Unknown"));

    let extra_tag_value = extra_tag.and_then(|t| {
        obj.element(t)
            .ok()
            .and_then(|e| e.to_str().ok().map(|s| s.to_string()))
    });

    Some(DicomInfo {
        series_description,
        file_path: file_path.to_path_buf(),
        extra_tag_value,
    })
}

pub fn group_by_series(files: Vec<DicomInfo>) -> HashMap<Arc<str>, Vec<DicomInfo>> {
    let mut groups: HashMap<Arc<str>, Vec<DicomInfo>> = HashMap::new();

    for info in files {
        groups
            .entry(Arc::clone(&info.series_description))
            .or_default()
            .push(info);
    }

    groups
}
