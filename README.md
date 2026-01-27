# dcs - DICOM Search CLI Tool

A command-line tool to search and group DICOM files by series.
It displays and searches through **Series Description**.

## Installation

```bash
cargo install --path .
```

## Usage

```bash
dcs <directory> [search]
```

### Arguments

- `<directory>` - The directory to scan for DICOM files (recursive)
- `[search]` (optional) - A search string

### Example

```bash
dcs patient_studies/ 

# With search
dcs patient_studies/ "soft tissue"
```
