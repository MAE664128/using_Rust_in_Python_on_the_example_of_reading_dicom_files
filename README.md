

# Finding DICOM files from Python code using Rust binary

## Description

**Educational code**

***Use python 3.11***

The code find and read all DICOM files at the specified path.

The code is implemented for educational purposes, to get acquainted with rust and pyO3

## Example: find and read all DICOM files

*(AMD Ryzen 5 6500X 6-Core Processor; Samsung SSD 970 EVO Plus 500TB)*
```commandline
Всего файлов в папке: 1972
──────────────────────────────────────────
| Python single process     | 8.45892s   |
| Rust called from python   | 1.13431s   |
| Python multiprocess.      | 2.13549s   |
──────────────────────────────────────────
```

*(AMD Ryzen 5 6500X 6-Core Processor; Samsung SSD 970 EVO Plus 500TB)*
```commandline
Всего файлов в папке: 66648
──────────────────────────────────────────
| Python single process     | 298.16290s |
| Rust called from python   |  41.99200s |
| Python multiprocess.      |  54.89347s |
──────────────────────────────────────────
```
