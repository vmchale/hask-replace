extern crate rayon;
extern crate colored;
extern crate walkdir;
extern crate regex;
#[macro_use]
extern crate clap;

use std::fs;
use rayon::prelude::*;
use clap::{App, AppSettings};
use std::path::PathBuf;
use walkdir::WalkDir;
use std::process::exit;
use colored::*;
use std::fs::File;
use std::ffi::OsStr;
use regex::Regex;
use std::io::prelude::*;

#[derive(Debug)]
struct ProjectOwned {
    pub dir: PathBuf,
    pub cabal_file: PathBuf
}

fn find_by_end_vec(ref p: &PathBuf, find: &str) -> Vec<PathBuf> {

    let s = p.to_string_lossy().to_string();

    let dir = WalkDir::new(&s)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|p| { 
            p.path().to_string_lossy().to_string().ends_with(find)
        });

    let vec: Vec<PathBuf> = dir.map(|x| x.path().to_path_buf()).collect();

    vec 
}


/// The arguments passed in here should *always* be strings pointing to directories
// TODO basically everything here could be faster.
fn get_cabal(p: PathBuf) -> ProjectOwned {

    let pre_parent = p.clone();
    let parent = pre_parent.parent().unwrap_or(&pre_parent); // default to itself when we can't find a parent
    let s = p.to_string_lossy().to_string();

    // FIXME finding cabal files shouldn't require recursion down arbitrarily many levels
    let vec = find_by_end_vec(&p, ".cabal");
    let vec_len = vec.len();

    // if we find more than one cabal file, abort.
    if vec_len > 1 {
        eprintln!("{}: more than one '.cabal' file in indicated directory, aborting.", "Error".red());
        exit(0x0001)
    } else if vec_len == 0 {
        ProjectOwned { dir: parent.to_path_buf(), cabal_file: p }
    } else {
        ProjectOwned { dir: PathBuf::from(s), cabal_file: { let mut cabal_path = p.clone() ; cabal_path.push("/test-nothing.cabal") ; cabal_path } } // FIXME wrong path to cabal file.
    }

}

pub fn get_dir(paths_from_cli: Option<&str>) -> &str {
    if let Some(read) = paths_from_cli {
        read
    } else {
        "."
    }
}

fn get_directory_contents(p: PathBuf) -> Vec<PathBuf> {

    let s = p.to_string_lossy().to_string();

    let dir = WalkDir::new(s).into_iter();

    let filtered = dir.filter_map(|e| e.ok()).filter(|p| {
        p.path().extension() == Some(OsStr::new("hs")) // TODO
    });

    filtered.map(|p| p.path().to_path_buf()).collect() // FIXME get rid of to_path_buf

}

fn module_to_file_name(module: &str) -> String {
    let mut replacements = module.replace(".","/");
    replacements.push_str(".hs");
    replacements
}

fn rayon_directory_contents(cabal: ProjectOwned, old_module: &str, new_module: &str) -> () {
    let dir: Vec<PathBuf> = get_directory_contents(cabal.dir);
    let iter = dir.into_par_iter().filter(|p| p.to_string_lossy().to_string().ends_with(".hs"));
    iter.for_each(|p| {
        println!("{:?}", p);
        let mut source_file = File::open(&p).unwrap();
        let mut source = String::new();
        source_file.read_to_string(&mut source).unwrap();
        let replacements = source.replace(old_module, new_module);
        let mut source_file_write = File::open(&p).unwrap();
        let _ = source_file_write.write_all(replacements.as_bytes());
    })
}

fn replace_all(cabal: ProjectOwned, old_module: &str, new_module: &str) -> () {

    // step 1: determine that the module we want to replace in fact exists
    let old_module_vec = find_by_end_vec(&cabal.dir, &module_to_file_name(old_module));
    let old_module_exists = !(old_module_vec.len() == 0);

    if !old_module_exists {
        println!("{:?}", old_module_vec);
        println!("{:?}", module_to_file_name(old_module));
        eprintln!("module '{}' does not exist in this project", old_module);
        exit(0x0001);
    }

    // TODO make this a method
    let mut cabal_string = cabal.dir.to_string_lossy().to_string();
    cabal_string.push_str(&cabal.cabal_file.to_string_lossy().to_string());

    let mut file = File::open(&cabal_string)
        .unwrap_or({ 
            eprintln!("file: {} failed to open", &cabal_string); 
            exit (0x0001) }
        );
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let re = Regex::new(old_module);

    let in_cabal_file = re.unwrap().is_match(&contents);

    if !in_cabal_file {
        eprintln!("module '{}' not found in your cabal file '{}'", old_module, &cabal_string);
        exit(0x0001);
    }

    // step 2: determine the targeted directory in fact exists, or make it ourselves.
    let vref: String = new_module.replace(".", "/");
    let mut v: Vec<&str> = vref.split('/').collect();
    v.pop();
    let target_directory = v.join("");
    if !PathBuf::from(&target_directory).exists() {
        let _ = fs::create_dir_all(&target_directory);
    }

    // step 3: replace the module in the '.cabal' file

    // step 4: replace every 'import Module' with 'import NewModule'
    rayon_directory_contents(cabal, old_module, new_module);

}

fn main() {
    let yaml = load_yaml!("options-en.yml");
    let matches = App::from_yaml(yaml)
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequired)
        .get_matches();

    // test stuff
    if let Some(command) = matches.subcommand_matches("function") {

        println!("function");

    } else if let Some(command) = matches.subcommand_matches("module") {

        let dir_string = get_dir(command.value_of("project"));

        let dir = PathBuf::from(dir_string);

        let old_module = command.value_of("old").unwrap();

        let new_module = command.value_of("new").unwrap();

        let cabal_project = get_cabal(dir);

        replace_all(cabal_project, old_module, new_module);

    } else {
        println!("nothing");
    }
}
