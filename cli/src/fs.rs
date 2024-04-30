use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub struct ReadFile {
    path: PathBuf,
    contents: String,
}

impl ReadFile {
    pub fn new(path: PathBuf, contents: String) -> ReadFile {
        ReadFile { path, contents }
    }

    pub fn get_path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn get_contents(&self) -> String {
        self.contents.clone()
    }
}

pub fn decompress_file(path: PathBuf) -> Vec<u8> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return Vec::new(),
    };

    let mut contents = Vec::new();

    return match file.read_to_end(&mut contents) {
        Ok(_) => {
            let mut decoder = brotli::Decompressor::new(&contents[..], 4096);
            let mut decompressed = Vec::new();
            match decoder.read_to_end(&mut decompressed) {
                Ok(_) => decompressed,
                Err(_) => Vec::new(),
            }
        }
        Err(_) => Vec::new(),
    };
}

#[allow(dead_code)]
pub fn read_file(path: PathBuf) -> Option<Vec<ReadFile>> {
    let mut file = match File::open(path.clone()) {
        Ok(file) => file,
        Err(_) => return None,
    };
    let mut contents = String::new();
    return match file.read_to_string(&mut contents) {
        Ok(_) => Some(Vec::from([ReadFile::new(path.clone(), contents)])),
        Err(_) => None,
    };
}

#[allow(dead_code)]
pub fn read_dir<T>(path: PathBuf, ext: &Vec<String>, f: fn(PathBuf) -> Option<Vec<T>>) -> Vec<T> {
    let mut contents = Vec::new();
    if path.is_dir() {
        for entry in path.read_dir().unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let mut child_contents: Vec<T> = read_dir(path, &ext, f);
            contents.append(&mut child_contents);
        }
    } else {
        let path_extension = path.extension().unwrap_or("".as_ref()).to_str();
        if path_extension.is_some() && ext.contains(&path_extension.unwrap().to_string()) {
            let mut child_contents: Vec<T> = f(path).unwrap();
            contents.append(&mut child_contents);
        }
    }

    return contents;
}

#[test]
fn check_decompress_file() {
    let path = PathBuf::from("tests/fixtures/.control-log.java.br");
    let contents = decompress_file(path);
    assert_eq!(String::from_utf8(contents).unwrap(), "\u{1}\0\0\0\0\0\0\0%\0\0\0\0\0\0\0cli/tests/resources/java/Program.java\u{14}\0\0\0\0\0\0\0/* control HE-110 */$\0\0\0\0\0\0\0System.out.println(\"Hello, World!\");\u{5}\0\0\0\0\0\0\0\u{8}\0\0\0\0\0\0\0\u{5}\0\0\0\0\0\0\0,\0\0\0\0\0\0\0".to_string());
}

#[test]
fn check_read_path() {
    let path = PathBuf::from("..");
    let extensions = vec!["rs".to_string()];
    let contents = read_dir(path, &extensions, read_file);
    assert!(contents.len() > 0);
}

#[test]
fn read_file_test() {
    let path = PathBuf::from("src/fs.rs");
    let content = read_file(path);
    assert!(content.is_some());
    assert!(content.unwrap().len() > 0);
}
