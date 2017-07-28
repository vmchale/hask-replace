extern crate rayon;
extern crate walkdir;
#[macro_use] extern crate clap;

use std::fs;
use rayon::prelude::*;
use clap::{App, AppSettings};
use std::path::PathBuf;
use rayon::iter::Map;
use walkdir::WalkDir;
use std::mem;
use std::ffi::OsStr;

pub fn get_dir(paths_from_cli: Option<&str>) -> &str {
    if let Some(read) = paths_from_cli {
        read
    } else {
        "."
    }
}

fn get_directory_contents(s: &str) -> Vec<PathBuf> {

    // step 1: determine if there are any files in the directory
    
    let dir = WalkDir::new(s).into_iter();

    let filtered = dir.filter_map(|e| e.ok()).filter(|p| p.path().extension() == Some(OsStr::new("rs"))); // TODO 

    filtered.map(|p| p.path().to_path_buf())
        .collect() // FIXME get rid of to_path_buf
} 

fn apply_transformation_dir(dir: Vec<PathBuf>) -> () {
    dir.into_par_iter()
         .for_each(|p| println!("{:?}", &p));
}

fn rayon_directory_contents(s: &str) -> () {
    apply_transformation_dir(get_directory_contents(s))
}

fn replace_all() -> () {
    // step 1: determine that the module we want to replace in fact exists
    
    // step 2: determine the targeted directory in fact exists, or make it ourselves.

    // step 3: replace the module in the '.cabal' file, or abort

    // step 4: replace every 'import Module' with 'import NewModule'
    
}

fn main() {
    let yaml = load_yaml!("options-en.yml");
    let matches = App::from_yaml(yaml)
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequired)
        .get_matches();

    // test stuff
    if let Some(command) = matches.subcommand_matches("function") {

        let dir = get_dir(command.value_of("project"));
        
        rayon_directory_contents(dir);

    }
    else if let Some(command) = matches.subcommand_matches("module") {

        let dir = get_dir(command.value_of("project"));
        
        rayon_directory_contents(dir);

    }
    else {
        println!("nothing");
    }
}
