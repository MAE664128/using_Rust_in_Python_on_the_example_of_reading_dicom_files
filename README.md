# BAD CODE
---

# Finding DICOM files from Python code using Rust binary

## Description

The code finds all DICOM files at the specified path.

The code is implemented for educational purposes, to get acquainted with rust and pyO3

## Example
*(AMD Ryzen 7 3700X 8-Core Processor; Samsung SSD 970 EVO Plus 1TB)*
```commandline
Total files in folder: 25545
┏───────────────────────────┬────────────┑
│ Python in one process     │ 32.93652   │
│ Rust call from python     │ 11.35903   │
│ Python multi process.     │ 8.91955    │
┕───────────────────────────┴────────────┙
```
