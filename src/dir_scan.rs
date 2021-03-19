
use std::path;
use std::convert;
use std::sync::{Arc, Mutex};
use walkdir::{DirEntry, WalkDir};
use rayon::prelude::*;

pub use dicom::object as dcm_object;
use std::ffi::OsString;
use std::collections::HashMap;
use std::path::PathBuf;
use dicom::object::{DefaultDicomObject, Error, RootDicomObject};
use dicom::object::mem::InMemDicomObject;
use dicom::dictionary_std::StandardDataDictionary;
use std::collections::hash_map::RandomState;



pub struct Scanner {
    root_path: String,
    load_tags: Vec<String>,
    tag_val_default: String,
    pub data: Option<HashMap<String, Option<HashMap<String, String>>>>,
}

impl Scanner {
    pub fn new(root_path: String, load_tags: Vec<String>, tag_val_default: String) -> Scanner {
        Scanner {
            root_path,
            load_tags,
            tag_val_default,
            data: None,
        }
    }

    pub fn get_root_path(&self) -> &String {
        &self.root_path
    }

    pub fn get_root_path_as_pathbuf(&self) -> path::PathBuf {
        path::PathBuf::from(&self.root_path)
    }

    pub fn get_load_tags(&self) -> &Vec<String> {
        return &self.load_tags;
    }

    pub fn get_tag_val_default(&self) -> &String {

        &self.tag_val_default

    }

    /// Возвращает вектор путей до файлов содержащихся в root
    /// Если ранее анализ папки не был выполнен, то будет запущено сканирование папки
    /// Иначе будут возвращены значения из cache
    pub fn paths(&mut self) -> Vec<String> {
        if self.data.is_none() { self.find_files().unwrap_or_else(|_| {}); }
        let data = &mut self.data;
        let mut paths: Vec<String> = Vec::new();
        match data {
            None => {}
            Some(el_data) => {
                for path in el_data.keys() {
                    paths.push(String::from(path.as_str()));
                };
            }
        }
        paths
    }

    pub fn data(&mut self) -> &Option<HashMap<String, Option<HashMap<String, String>>>> {
        &self.data
    }

    /// Выплнить рекурсивный поск всех файлов в root_path
    fn find_files(&mut self) -> Result<(), ()> {
        let mut value: HashMap<String, Option<HashMap<String, String>>> = HashMap::new();
        let path = self.get_root_path_as_pathbuf();
        let paths = find_all_files(&path);
        for path in paths {
            let tmp_path = match path.to_str() {
                None => { continue; }
                Some(path) => { path.to_string() }
            };
            value.insert(tmp_path, None);
        };
        self.data = Some(value);
        Ok(())
    }

    /// Выполнить попытку чтения всех файлов по пути root_path
    /// Если файл является DICOM, то информация о нем будет добавлена в data
    pub fn read_files(&mut self) -> &Option<HashMap<String, Option<HashMap<String, String>>>> {
        if self.data.is_none() { self.find_files().unwrap_or_else(|_| {}); }
        let load_tags = self.get_load_tags();
        let tag_val_default = self.get_tag_val_default();

        let new_data: HashMap<String, Option<HashMap<String, String>>> = HashMap::new();
        let contents = Arc::new(Mutex::new(new_data));
        match &self.data {
            None => {}
            Some(paths_and_tags) => {
                paths_and_tags.into_par_iter()
                    .for_each(|(key, _)| {
                        let path = path::PathBuf::from(key);
                        let dcm_tags = load_dcm_file(
                            path.to_path_buf(), load_tags, &tag_val_default);

                        add_data_content(&contents, (String::from(key), dcm_tags));
                    });
            }
        }
        let new_data = match contents.lock() {
            Ok(c) => {
                Some(c.to_owned())
            }
            Err(_) => {
                None
            }
        };
        self.data = new_data;
        &self.data
    }
}

fn add_data_content(contents: &Arc<Mutex<HashMap<String, Option<HashMap<String, String>>>>>,
                    data: (String, Option<HashMap<String, String>>)) {
    match contents.lock() {
        Ok(mut c) => {
            c.insert(data.0, data.1);
        }
        Err(_) => {}
    }
}


/// Проверяет, является ли директория скрытой
fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

/// Выполняет рекурсивный поиск всех файлов в директории
/// Поиск не выполняется в скрытых директориях
/// Возвращает вектор путей
pub fn find_all_files(dir_path: &path::PathBuf) -> Vec<path::PathBuf> {
    WalkDir::new(dir_path)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok())
        .map(|file| file.path().to_owned())
        .collect()
}

/// Возвращает значения тегов из DICOM объекта, в виде HashMap<k,v>
pub fn load_tags_of_dcm<'a>(dcm_obj: DefaultDicomObject, load_tags: &Vec<String>,
                            tag_val_default: &str) -> HashMap<String, String> {
    let mut tags: HashMap<String, String> = HashMap::new();
    for tag_name in load_tags {
        let mut new_val: String = String::new();
        match dcm_obj.element_by_name(&tag_name) {
            Ok(item) => {
                match item.to_multi_str() {
                    Ok(val) => {
                        let values = val.to_vec();
                        if values.len() == 1 {
                            new_val.push_str(&values[0].trim().trim_matches('\x00').to_string())
                        } else if values.len() == 0 {
                            new_val.push_str("");
                        } else {
                            new_val.push_str("[");
                            new_val.push_str(&values.join(", ").trim().trim_matches('\x00'));
                            new_val.push_str("]");
                        };
                    }
                    Err(_) => {
                        new_val.push_str(tag_val_default);
                    }
                }
            }
            Err(_) => {
                new_val.push_str(tag_val_default);
            }
        }
        tags.insert(tag_name.to_string().to_owned(), new_val.trim().trim_matches('\x00').to_string());
    }
    tags
}

/// Пытается прочитать DICOM файл и в качестве успеха возвращает tags HashMap<k,v>
/// Иначе возвращает None
pub fn load_dcm_file(path: path::PathBuf, load_tags: &Vec<String>,
                     tag_val_default: &str) -> Option<HashMap<String, String>> {
    match dcm_object::open_file(&path.as_path()) {
        Ok(dcm_obj) => {
            Some(load_tags_of_dcm(dcm_obj, load_tags, tag_val_default))
        }
        Err(_) => { None }
    }
}
