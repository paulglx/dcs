# dcs - DICOM Search CLI Tool

A command-line tool to search and group DICOM files by series.

## Installation

```bash
cargo build --release
```

The binary will be available at `target/release/dcs`.

## Usage

```bash
dcs <directory>
```

### Arguments

- `<directory>` - The directory to scan for DICOM files (recursive)

### Example

```bash
dcs /path/to/dicom/files
```

### Output

The tool will scan the specified directory recursively for DICOM files and group them by series description. For each series, it displays:

- **Series Description** - The name of the series
- **Patient ID** - The patient identifier (tag 0010,0020)
- **Study Description** - The study description (tag 0008,1030)
- **Files** - Number of files and path to the first file

Example output:

```
Scanning directory: /path/to/dicom/files

Series: T1 AXIAL
  Patient ID: PATIENT001
  Study: Brain MRI
  Files: 120 (first: /path/to/dicom/files/IM001.dcm)

Series: T2 FLAIR
  Patient ID: PATIENT001
  Study: Brain MRI
  Files: 60 (first: /path/to/dicom/files/IM121.dcm)
```

## Dependencies

- [clap](https://crates.io/crates/clap) - CLI argument parsing
- [dicom-rs](https://crates.io/crates/dicom) - DICOM file parsing
- [walkdir](https://crates.io/crates/walkdir) - Recursive directory traversal
