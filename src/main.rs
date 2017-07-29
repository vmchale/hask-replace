#[macro_use]
extern crate clap;
extern crate rayon;
extern crate colored;
extern crate walkdir;

use std::fs;
use rayon::prelude::*;
use clap::{App, AppSettings};
use std::path::PathBuf;
use walkdir::WalkDir;
use std::process::exit;
use colored::*;
use std::fs::File;
use std::io::prelude::*;
use std::fs::OpenOptions;

#[derive(Debug)]
struct ProjectOwned {
    pub dir: PathBuf,
    pub cabal_file: PathBuf,
}

impl ProjectOwned {
    fn get_cabal_path(&self) -> String {
        get_cabal(&self.dir)
            .cabal_file
            .to_string_lossy()
            .to_string()
    }
}


fn find_by_end_vec(p: &PathBuf, find: &str, depth: Option<usize>) -> Vec<PathBuf> {

    let s = p.to_string_lossy().to_string();

    let dir = if let Some(d) = depth {
        WalkDir::new(&s).max_depth(d)
    } else {
        WalkDir::new(&s)
    };
    let iter = dir.into_iter().filter_map(|e| e.ok()).filter(|p| {
        let path = p.path();
        (!path.starts_with(".")) && path.file_name().unwrap().to_string_lossy().to_string().ends_with(find)
    });

    let vec: Vec<PathBuf> = iter.map(|x| x.path().to_path_buf()).collect();

    vec
}


/// The arguments passed in here should *always* be strings pointing to directories
fn get_cabal(p: &PathBuf) -> ProjectOwned {

    let parent = p.parent().unwrap_or(p);
    let s = p.to_string_lossy().to_string();

    let mut vec = find_by_end_vec(p, ".cabal", Some(1));
    let vec_len = vec.len();

    // if we find more than one cabal file, abort.
    if vec_len > 1 {
        eprintln!(
            "{}: more than one '.cabal' file in indicated directory, aborting.",
            "Error".red()
        );
        exit(0x0001)
    } else if vec_len == 0 {
        ProjectOwned {
            dir: parent.to_path_buf(),
            cabal_file: p.clone(),
        }
    } else {
        let cabal_name = vec.pop().unwrap();
        ProjectOwned {
            dir: PathBuf::from(s),
            cabal_file: {
                cabal_name
            },
        }
    }

}

pub fn get_dir(paths_from_cli: Option<&str>) -> &str {
    if let Some(read) = paths_from_cli {
        read
    } else {
        "."
    }
}

fn get_source_files(p: &PathBuf) -> Vec<PathBuf> {

    let s = p.to_string_lossy().to_string();

    let dir = WalkDir::new(s).into_iter();

    let filtered = dir.filter_map(|e| e.ok()).filter(|p| {
        let path = p.path();
        !path.starts_with(".") && p.file_name().to_string_lossy().to_string().ends_with(".hs")
    });

    filtered.map(|p| p.path().to_path_buf()).collect()

}

fn module_to_file_name(module: &str) -> String {
    let mut replacements = module.replace(".", "/");
    replacements.push_str(".hs");
    replacements
}

fn rayon_directory_contents(cabal: &ProjectOwned, old_module: &str, new_module: &str) -> () {
    let dir: Vec<PathBuf> = get_source_files(&cabal.dir);
    let iter = dir.into_par_iter().filter(|p| {
        !p.starts_with(".") && p.file_name().unwrap().to_string_lossy().to_string().ends_with(".hs")
    });
    iter.for_each(|p| {
        let mut source_file = File::open(&p).unwrap();
        let mut source = String::new();
        source_file.read_to_string(&mut source).unwrap();
        let replacements = source.replacen(old_module, new_module, 1);
        let mut source_file_write = File::create(&p).unwrap();
        let _ = source_file_write.write(replacements.as_bytes());
    })
}

fn replace_all(cabal: &ProjectOwned, old_module: &str, new_module: &str) -> () {

    // step 1: determine that the module we want to replace in fact exists
    let mut old_module_vec = find_by_end_vec(&cabal.dir, &module_to_file_name(old_module), None);
    let old_module_exists = !(old_module_vec.is_empty());

    let (old_module_name, src_dir) = if old_module_exists {
        let name = old_module_vec.pop().unwrap();
        let name_string: String = name.to_string_lossy().to_string();
        let name_str: &str = name_string.as_str();
        let old_string: String = module_to_file_name(old_module);
        let old_str: &str = old_string.as_str();
        let dir: &str = name_str.trim_right_matches(old_str);
        (name, dir.to_string())
    } else {
        println!("{:?}", old_module_vec);
        eprintln!("module '{}' does not exist in this project", old_module);
        exit(0x0001);
    };

    // TODO make this a method
    let cabal_string = cabal.get_cabal_path();

    let mut cabal_file = match File::open(&cabal_string) {
        Ok(x) => x,
        _ => {
            eprintln!(
                "{}: Failed to open file at: {}",
                "Error".red(),
                cabal_string
            );
            exit(0x0001)
        }
    };
    let mut contents = String::new();
    match cabal_file.read_to_string(&mut contents) {
        Ok(_) => (),
        _ => {
            eprintln!("{}: Failed to read file at: {}", "Error".red(), contents);
            exit(0x0001)
        }
    }

    let in_cabal_file = (&contents).contains(old_module);

    if !in_cabal_file {
        eprintln!(
            "module '{}' not found in your cabal file '{}'",
            old_module,
            &cabal_string
        );
        exit(0x0001);
    }

    // step 2: determine the targeted directory in fact exists, or make it ourselves.
    let vref: String = new_module.replace(".", "/");
    let v: Vec<&str> = vref.split('/').collect();
    let mut target_directory: String = src_dir.clone();
    target_directory.push_str(&v.join("/"));
    if !PathBuf::from(&target_directory).exists() {
        match fs::create_dir_all(&target_directory) {
            Ok(x) => x,
            _ => {
                eprintln!(
                    "{}: failed to create directory '{}'",
                    "Error".red(),
                    target_directory
                );
                exit(0x0001)
            }
        }
    }

    // step 3: replace the module in the '.cabal' file
    let p: PathBuf = PathBuf::from(&cabal_string);
    let mut source_file = match OpenOptions::new().read(true).open(&p) {
        Ok(x) => x,
        _ => {
            eprintln!("{}: Failed to open file at: {}", "Error".red(), p.display());
            exit(0x0001)
        }
    };
    let mut source = String::new();
    match source_file.read_to_string(&mut source) {
        Ok(_) => (),
        _ => {
            eprintln!("{}: Failed to read file at: {}", "Error".red(), p.display());
            exit(0x0001)
        }
    }
    let replacements = source.replace(old_module, new_module);
    let mut source_file_write = OpenOptions::new().write(true).open(&p).unwrap();
    match source_file_write.write(replacements.as_bytes()) {
        Ok(_) => (),
        _ => {
            eprintln!(
                "{}: Failed to write file at: {}",
                "Error".red(),
                p.display()
            );
            exit(0x0001)
        }
    }

    // step 4: replace every 'import Module' with 'import NewModule'
    rayon_directory_contents(cabal, old_module, new_module);

    // step 5: move the actual file
    let mut new_module_path = src_dir;
    new_module_path.push_str(&module_to_file_name(new_module));
    if let Ok(s) = fs::rename(&old_module_name, &new_module_path) {
        s
    } else {
        eprintln!(
            "{}: failed to rename module {} to {}",
            "Error".red(),
            old_module_name.to_string_lossy(),
            new_module_path
        );
        exit(0x0001)
    }

}

fn main() {
    let yaml = load_yaml!("options-en.yml");
    let matches = App::from_yaml(yaml)
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequired)
        .get_matches();

    if let Some(command) = matches.subcommand_matches("function") {

        eprintln!("not yet implemented");

    } else if let Some(command) = matches.subcommand_matches("module") {

        let dir_string = get_dir(command.value_of("project"));

        let dir = PathBuf::from(dir_string);

        let old_module = command.value_of("old").unwrap(); // okay because a subcommand is required

        let new_module = command.value_of("new").unwrap(); // okay beacause a subcommand is required

        let cabal_project = get_cabal(&dir);

        replace_all(&cabal_project, old_module, new_module);

    } else {
        eprintln!("{}: failed to supply a subcommand", "Error".red());
    }
}
