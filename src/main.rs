#[macro_use]
extern crate clap;
#[macro_use]
extern crate text_io;
extern crate rayon;
extern crate colored;
extern crate regex;
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
use regex::{Regex, Captures};
use std::process::Command;
use std::path::Path;
use std::fmt::Debug;

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
        (!path.starts_with(".stack-work")) && (path.to_string_lossy().to_string().ends_with(find))
    });

    let vec: Vec<PathBuf> = iter.map(|x| x.path().to_path_buf()).collect();

    vec
}


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

fn get_dir(paths_from_cli: Option<&str>) -> &str {
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
        !path.starts_with(".stack-work") &&
            p.file_name().to_string_lossy().to_string().ends_with(".hs")
    });

    filtered.map(|p| p.path().to_path_buf()).collect()

}

fn module_to_file_name(module: &str) -> String {
    let mut replacements = module.replace(".", "/");
    replacements.push_str(".hs");
    replacements
}

fn get_yes() -> bool {
    let s: String = read!("[y/n]: {}");
    let s2: &str = &s;
    match s2 {
        "Yes" | "Y" | "y" | "yes" => true,
        _ => false,
    }
}

fn rayon_directory_contents(cabal: &ProjectOwned, old_module: &str, new_module: &str) -> () {
    let dir: Vec<PathBuf> = get_source_files(&cabal.dir);
    // FIXME we filter the directory results twice
    let iter = dir.into_par_iter().filter(|p| {
        !p.starts_with(".stack-work") &&
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
                .ends_with(".hs")
    });
    iter.for_each(|p| {
        let mut source_file = File::open(&p).unwrap();
        let mut source = String::new();
        source_file.read_to_string(&mut source).unwrap();
        let mut old_module_regex = "(".to_string();
        old_module_regex.push_str(&old_module.replace(".", "\\."));
        old_module_regex.push_str(")+");
        let mut old_module_regex = old_module.to_string();
        old_module_regex.push_str("(\n|\\(|( *) \\(|( *) where|\\.)+?");
        let re = Regex::new(&old_module_regex).unwrap();
        let replacements = re.replacen(&source, 0, |caps: &Captures| {
            format!("{}{}", new_module, &caps[1])
        }).to_string();
        write_file(&p, replacements);
    })
}

fn write_file<P: AsRef<Path> + Debug>(p: P, s: String) -> () {

    let mut file = match File::create(&p) {
        Ok(x) => x,
        _ => {
            eprintln!("{}: Failed to open file at: {:?}", "Error".red(), p);
            exit(0x0001)
        }
    };
    match file.write(s.as_bytes()) {
        Ok(_) => (),
        _ => {
            eprintln!("{}: Failed to write file at: {:?}", "Error".red(), p);
            exit(0x0001)
        }
    }

}

fn read_file<P: AsRef<Path> + Debug>(p: P) -> String {

    let mut file = match File::open(&p) {
        Ok(x) => x,
        _ => {
            eprintln!("{}: Failed to open file at: {:?}", "Error".red(), p);
            exit(0x0001)
        }
    };
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => (),
        _ => {
            eprintln!("{}: Failed to read file at: {:?}", "Error".red(), contents);
            exit(0x0001)
        }
    }

    contents
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
        eprintln!("module '{}' does not exist in this project", old_module);
        exit(0x0001);
    };

    // TODO make this a method
    let cabal_string = cabal.get_cabal_path();

    let contents = read_file(&cabal_string);

    let in_cabal_file = (&contents).contains(old_module);

    if !in_cabal_file {
        eprintln!(
            "{}: module '{}' not found in your cabal file '{}'",
            "Warning".yellow(),
            old_module,
            &cabal_string
        );
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
    let source = read_file(&cabal_string);
    let mut old_module_regex = "(".to_string();
    old_module_regex.push_str(&old_module.replace(".", "\\."));
    old_module_regex.push_str(")+");
    let mut old_module_regex = old_module.to_string();
    old_module_regex.push_str("(\n|,)+?");
    let re = Regex::new(&old_module_regex).unwrap();
    let replacements = re.replacen(&source, 2, |caps: &Captures| {
        format!("{}{}", new_module, &caps[1])
    }).to_string();

    write_file(&cabal_string, replacements);

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

fn git_commit(src_dir: &str) -> () {
    let mut cmd = "cd ".to_string();
    cmd.push_str(src_dir);
    cmd.push_str("&&");
    cmd.push_str("git commit -am 'automatic commit made by hask-replace'");
    if let Ok(c) = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .spawn()
    {
        c.wait_with_output().expect("failed to wait on child");
    } else {
        eprintln!("{}, git failed to stash changes. Aborting.", "Error".red());
        exit(0x0001);
    }
}

fn module_exists(cabal: &ProjectOwned, module: &str) -> (PathBuf, String) {

    // step 1: determine that the module to find the function in actually exists
    let mut module_vec = find_by_end_vec(&cabal.dir, &module_to_file_name(module), None);
    let module_exists = !(module_vec.is_empty());

    let (module_name, src_dir) = if module_exists {
        let name = module_vec.pop().unwrap();
        let name_string: String = name.to_string_lossy().to_string();
        let name_str: &str = name_string.as_str();
        let old_string: String = module_to_file_name(module);
        let old_str: &str = old_string.as_str();
        let dir: &str = name_str.trim_right_matches(old_str);
        (name, dir.to_string())
    } else {
        eprintln!("module '{}' does not exist in this project", module);
        exit(0x0001);
    };

    (module_name, src_dir)
}


fn move_function(cabal: &ProjectOwned, function: &str, old_module: &str, new_module: &str) {

    // step 1: confirm the modules exist
    let (old_module_path, _) = module_exists(cabal, old_module);
    let (new_module_path, _) = module_exists(cabal, new_module);

    // step 2: move the actual function

    // create the regex for the (top-level) function
    let mut regex_str: String = "\n".to_string();
    regex_str.push_str(function);
    regex_str.push_str("( *::.*\n)?");
    regex_str.push_str(function);
    regex_str.push_str("(.*\n)?");
    let re = Regex::new(&regex_str).unwrap();

    // write the stuff
    let old = read_file(&old_module_path);
    let captures = re.find(&old).unwrap(); // FIXME bad!!
    let (i, j) = (captures.start(), captures.end());
    let func_str = &old[i..j];
    let mut new = read_file(&new_module_path);
    new.push_str(func_str);
    write_file(new_module_path, new);
    let mut old_write = (&old[..i]).to_string(); // TODO check this slice on byte indices
    old_write.push_str(&old[j..]);
    write_file(old_module_path, old_write);

    // step 3: remove the function from the list of explicit exports of the first module if
    // necessary, and add it to the list of explicit exports of the second if necessary

    // step 4: anywhere that the old module was imported, add the new module, if the function
    // is called in that module. If it was imported explicitly, import it explicitly unless the new
    // module is already there. If the old module's explicit imports are empty now, warn the user
    // in case they still need the instances from the old module.
    eprintln!(
        "{}: hr does not yet replace explicit and qualified imports across projects!",
        "Warning".yellow()
    );

    // step 5: if the old module was imported under a qualified name, and the function was called
    // using this qualified name, import the new module qualified (if it's not already imported)
    // using the first letter of the last bit to name it, and then replace the qualified uses of
    // the old function

}

fn main() {
    let yaml = load_yaml!("options-en.yml");
    let matches = App::from_yaml(yaml)
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequired)
        .get_matches();

    if let Some(command) = matches.subcommand_matches("function") {

        let dir_string = get_dir(command.value_of("project"));

        let dir = PathBuf::from(dir_string);

        let old_module = command.value_of("old").unwrap(); // okay because a subcommand is required

        let new_module = command.value_of("new").unwrap(); // okay beacause a subcommand is required

        let cabal_project = get_cabal(&dir);

        if command.is_present("stash") {
            git_commit(&cabal_project.dir.to_string_lossy().to_string());
        }

        let function = command.value_of("function").unwrap();

        move_function(&cabal_project, function, old_module, new_module);

    } else if let Some(command) = matches.subcommand_matches("module") {

        let dir_string = get_dir(command.value_of("project"));

        let dir = PathBuf::from(dir_string);

        let old_module = command.value_of("old").unwrap(); // okay because a subcommand is required

        let new_module = command.value_of("new").unwrap(); // okay beacause a subcommand is required

        let cabal_project = get_cabal(&dir);

        if command.is_present("stash") {
            git_commit(&cabal_project.dir.to_string_lossy().to_string());
        }

        replace_all(&cabal_project, old_module, new_module);

    } else {
        eprintln!("{}: failed to supply a subcommand", "Error".red());
    }
}
