extern crate walkdir;
extern crate regex;
 
use regex::Regex;
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use walkdir::WalkDir;
 
fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        // no arguments passed (only program itself)
        1 => {
            println!("Usage: cargo run index <map_directory> <source_directory>*");
            println!("Usage: cargo run search <term>*");
        },
        _ => {
            let command         = &args[1];
            let directory       = args[2].to_string();
            let params          = args[3..].to_vec();
            let mut map         = PersistentMultiMap::new(directory);
            match &command[..] {
                "index" => { map.create_index(params); }
                "search" => { map.search(params); }
                _ => { println!("Usage: unknown command: {}", command) }
            }
        }
    }
}
 
fn walk_files(directories: Vec<String>, visitor: &mut PathVisitor) {
    let mut progress = 0;
    for dir in directories {
        for entry in WalkDir::new(dir) {
            if let Ok(dir_entry) = entry {
                if let Ok(metadata) = dir_entry.metadata() {
                    if metadata.is_file() {
                        progress += 1;
                        visitor.visit(progress, dir_entry.path());
                    }
                }
            }
        }
    }
}
 
trait PathVisitor {
    fn visit(&mut self, progress: u32, path: &Path);
}
 
// ----------------------------------------------------------------------
 
struct PersistentMultiMap {
    directory: PathBuf
}
 
impl PersistentMultiMap {
    fn search(&self, terms: Vec<String>) {
        let entry = terms.iter().next();
        if let Some(first_term) = entry {
            let mut result = self.get(first_term);
            for term in terms.iter().skip(1) {
                let values = self.get(term);
                result = result.intersection(&values).cloned().collect();
            }
            for value in result {
                println!("{}", value);
            }
        }
    }
 
    fn create_index(&mut self, source_directories: Vec<String>) {
        walk_files(source_directories, self);
        self.summarize()
    }
 
    fn new(directory: String) -> PersistentMultiMap {
        fs::create_dir_all(&directory).expect(&format!("directory cannot be created: {}", directory)); 
        PersistentMultiMap{ directory: PathBuf::from(directory) }
    }
 
    fn get_path(&self, key: &str) -> PathBuf {
        self.directory.join(key.to_lowercase())
    }
 
    fn get(&self, key: &str) -> HashSet<String> {
        let mut values = HashSet::new();
        let key_path = self.get_path(&key);
        if key_path.exists() {
            let reader = BufReader::new(File::open(&key_path).expect(&format!("key file does not exist: {}", key)));
            for line in reader.lines() {
                if let Ok(value) = line {
                    values.insert(value);
                }
            }
        }
        values
    }
 
    fn add(&self, key: &str, value: &str) {
        match OpenOptions::new().create(true).append(true).open(&self.get_path(key)) {
            Ok(mut key_file) => {
                if let Err(error) = write!(key_file, "{}\n", value) {
                    println!("Warning: could not write to: {}: {}", key, error);
                }
            }
            Err(error) => {
                println!("Warning: could not append to: {}: {}", key, error);
            }
        }
    }
 
    fn summarize(&self) {
        println!("keys: {}", fs::read_dir(&self.directory).expect(&format!("directory cannot be read: {}", self.directory.to_str().unwrap())).count())
    }
}    
 
impl PathVisitor for PersistentMultiMap {
 
    fn visit(&mut self, progress: u32, path: &Path) {
        if progress % 100 == 0 {
            println!("progress = {}", progress);
        }
        if let Some(source_word_file) = path.to_str() {
            let mut source_file = File::open(path).expect(&format!("source file does not exist: {}", source_word_file));
            let mut source_text = String::new();
            if source_file.read_to_string(&mut source_text).is_ok() {
                let re = Regex::new(r"\W+").expect("illegal regex");
                let source_words: HashSet<&str> = re.split(&source_text).collect();
                for source_word in source_words {
                    if ! source_word.is_empty() {
                        self.add(source_word, source_word_file);
                    }
                }
            }
        }
    }
}
