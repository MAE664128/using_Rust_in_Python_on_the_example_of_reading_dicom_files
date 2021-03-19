use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

mod dir_scan;

use std::path;
use std::collections::HashMap;
use std::borrow::{Cow, Borrow};
use rayon::prelude::*;

#[pyfunction]
fn find_dcm_file_paths(path: String) -> PyResult<Vec<String>> {
    let test:Vec<String> = Vec::new();
    let mut scanner = dir_scan::Scanner::new(
        path, test, "*NO_TAG*".to_string());
    Ok(scanner.paths())
}

#[pyfunction]
fn load_dcm_files_in_dir<'a>(path: String, load_tags: Vec<String>,
                             tag_val_default: String) -> PyResult<Option<HashMap<String, Option<HashMap<String, String>>>>> {

    let mut scanner = dir_scan::Scanner::new(path,
    load_tags,tag_val_default);
    scanner.read_files();
    Ok(scanner.data)
}


#[pymodule]
fn py_dcm_finder_rs(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(find_dcm_file_paths, m)?)?;
    m.add_function(wrap_pyfunction!(load_dcm_files_in_dir, m)?)?;
    Ok(())
}


