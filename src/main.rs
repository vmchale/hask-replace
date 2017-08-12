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
// TODO should encapsulate strings!
struct ProjectOwned {
    pub copy: bool,
    pub dir: PathBuf,
    pub config_file: PathBuf,
    pub module_extension: String,
    pub config_extension: String,
}

impl ProjectOwned {
    fn get_config_path(&self, module_ext: &str, config_ext: &str) -> String {
        get_config(&self.dir, module_ext, config_ext, self.copy)
            .config_file
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


fn get_config(p: &PathBuf, module_ext: &str, config_ext: &str, copy: bool) -> ProjectOwned {

    let parent = p.parent().unwrap_or(p);
    let s = p.to_string_lossy().to_string();

    let vec = find_by_end_vec(p, config_ext, Some(1));
    let vec_len = vec.len();

    // if we find more than one config file, abort.
    if vec_len > 1 && config_ext == ".config" {
        eprintln!(
            "{}: more than one '{}' file in indicated directory, aborting.",
            config_ext,
            "Error".red()
        );
        exit(0x0001)
    } else if vec_len == 0 {
        ProjectOwned {
            copy: copy,
            dir: parent.to_path_buf(),
            config_file: p.clone(),
            module_extension: module_ext.to_string(),
            config_extension: config_ext.to_string(),
        }
    } else {
        let config_name = vec.into_iter()
            .filter(|p| p.to_string_lossy() != "test.ipkg")
            .collect::<Vec<PathBuf>>()
            .pop()
            .unwrap();
        ProjectOwned {
            copy: copy,
            dir: PathBuf::from(s),
            config_file: {
                config_name
            },
            module_extension: module_ext.to_string(),
            config_extension: config_ext.to_string(),
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

fn get_source_files(p: &PathBuf, extension: &str) -> Vec<PathBuf> {

    let s = p.to_string_lossy().to_string();

    let dir = WalkDir::new(s).into_iter();

    let filtered = dir.filter_map(|e| e.ok()).filter(|p| {
        let path = p.path();
        !path.starts_with(".stack-work") &&
            p.file_name().to_string_lossy().to_string().ends_with(
                extension,
            )
    });

    filtered.map(|p| p.path().to_path_buf()).collect()

}

fn module_to_file_name(module: &str, extension: &str) -> String {
    let mut replacements = module.replace(".", "/");
    replacements.push_str(extension);
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

fn rayon_directory_contents(
    config: &ProjectOwned,
    old_module: &str,
    new_module: &str,
    extension: &str,
) -> () {
    let dir: Vec<PathBuf> = get_source_files(&config.dir, extension);
    // FIXME we filter the directory results twice
    let iter = dir.into_par_iter();
    iter.for_each(|p| {
        let mut source_file = File::open(&p).unwrap();
        let mut source = String::new();
        source_file.read_to_string(&mut source).unwrap();
        let mut old_module_regex = "(".to_string();
        old_module_regex.push_str(&old_module.replace(".", "\\."));
        old_module_regex.push_str(")+");
        let mut old_module_regex = old_module.to_string();
        old_module_regex.push_str("(\n|( +)exposing.*\n|( +)\\(|( +)where)+?");
        let re = Regex::new(&old_module_regex).unwrap();
        let num = if extension == ".idr" { 1 } else { 0 }; // FIXME
        let replacements = re.replacen(&source, num, |caps: &Captures| {
            format!("{}{}", new_module, &caps[1])
        }).to_string();
        write_file(&p, &replacements);
    })
}

fn write_file<P: AsRef<Path> + Debug>(p: P, s: &str) -> () {

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

fn replace_all(config: &ProjectOwned, old_module: &str, new_module: &str) -> () {

    let module_ext: &str = &config.module_extension;
    let config_ext: &str = &config.config_extension;

    // step 1: determine that the module we want to replace in fact exists
    let mut old_module_vec = find_by_end_vec(
        &config.dir,
        &module_to_file_name(old_module, module_ext),
        None,
    );
    let old_module_exists = !(old_module_vec.is_empty());

    let (old_module_name, src_dir) = if old_module_exists {
        let name = old_module_vec.pop().unwrap();
        let name_string: String = name.to_string_lossy().to_string();
        let name_str: &str = name_string.as_str();
        let old_string: String = module_to_file_name(old_module, module_ext);
        let old_str: &str = old_string.as_str();
        let dir: &str = name_str.trim_right_matches(old_str);
        (name, dir.to_string())
    } else {
        eprintln!("module '{}' does not exist in this project", old_module);
        exit(0x0001);
    };

    // TODO make this a method
    let config_string = config.get_config_path(module_ext, config_ext);

    let contents = read_file(&config_string);

    let in_config_file = (&contents).contains(old_module);

    if !in_config_file {
        eprintln!(
            "{}: module '{}' not found in your config file '{}'",
            "Warning".yellow(),
            old_module,
            &config_string
        );
    }

    // step 2: determine the targeted directory in fact exists, or make it ourselves.
    let vref: String = new_module.replace(".", "/");
    let pre_v: Vec<&str> = vref.split('/').collect();
    let l = (&pre_v).len();
    let v: Vec<&str> = pre_v.into_iter().take(l - 1).collect();
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
    let source = read_file(&config_string);
    let mut old_module_regex = "(".to_string();
    old_module_regex.push_str(&old_module.replace(".", "\\."));
    old_module_regex.push_str(")+");
    let mut old_module_regex = old_module.to_string();
    old_module_regex.push_str("(\n|,)+?");
    let re = Regex::new(&old_module_regex).unwrap();
    let replacements = if !config.copy {
        re.replacen(&source, 2, |caps: &Captures| {
            format!("{}{}", new_module, &caps[1])
        }).to_string()
    } else {
        re.replacen(&source, 2, |caps: &Captures| {
            format!("{}{}, {}{}", new_module, &caps[1], old_module, &caps[1])
        }).to_string()
    };

    write_file(&config_string, &replacements);

    // step 4: replace every 'import Module' with 'import NewModule'
    rayon_directory_contents(config, old_module, new_module, module_ext);

    // TODO copy a file only?
    // step 5: move the actual file
    let mut new_module_path = src_dir;
    new_module_path.push_str(&module_to_file_name(new_module, module_ext));
    if Path::new(&new_module_path).exists() {
        eprintln!("{}: destination module already exists.", "Error".red());
        exit(0x0001);
    }

    let expr = if !config.copy {
        fs::rename(&old_module_name, &new_module_path)
    } else {
        fs::copy(&old_module_name, &new_module_path).map(|_| ())
    };

    if let Ok(s) = expr {
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

fn module_exists(config: &ProjectOwned, module: &str, extension: &str) -> (PathBuf, String) {

    // step 1: determine that the module to find the function in actually exists
    let mut module_vec =
        find_by_end_vec(&config.dir, &module_to_file_name(module, extension), None);
    let module_exists = !(module_vec.is_empty());

    let (module_name, src_dir) = if module_exists {
        let name = module_vec.pop().unwrap();
        let name_string: String = name.to_string_lossy().to_string();
        let name_str: &str = name_string.as_str();
        let old_string: String = module_to_file_name(module, extension);
        let old_str: &str = old_string.as_str();
        let dir: &str = name_str.trim_right_matches(old_str);
        (name, dir.to_string())
    } else {
        eprintln!("module '{}' does not exist in this project", module);
        exit(0x0001);
    };

    (module_name, src_dir)
}


fn move_function(config: &ProjectOwned, function: &str, old_module: &str, new_module: &str) {

    // step 1: confirm the modules exist
    let (old_module_path, _) = module_exists(config, old_module, &config.config_extension);
    let (new_module_path, _) = module_exists(config, new_module, &config.config_extension);

    // step 2: move the actual function

    // create the regex for the (top-level) function
    let mut regex_str: String = "\n".to_string();
    regex_str.push_str(function);
    regex_str.push_str("( *:.*\n)?");
    regex_str.push_str(function);
    regex_str.push_str("(.*\n)?");
    let re = Regex::new(&regex_str).unwrap();

    // use nom for find and replace??

    // write the stuff
    let old = read_file(&old_module_path);
    let captures = re.find(&old).unwrap(); // FIXME bad!!
    let (i, j) = (captures.start(), captures.end());
    let func_str = &old[i..j];
    let mut new = read_file(&new_module_path);
    new.push_str(func_str);
    write_file(new_module_path, &new);
    let mut old_write = (&old[..i]).to_string(); // TODO check this slice on byte indices
    old_write.push_str(&old[j..]);
    write_file(old_module_path, &old_write);

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

        let extension = if command.is_present("hpack") {
            ".yaml"
        } else {
            ".cabal"
        };

        let config_project = get_config(&dir, ".hs", &extension, command.is_present("copy"));

        if command.is_present("stash") {
            git_commit(&config_project.dir.to_string_lossy().to_string());
        }

        let function = command.value_of("function").unwrap();

        move_function(&config_project, function, old_module, new_module);

    } else if let Some(command) = matches.subcommand_matches("module") {

        let dir_string = get_dir(command.value_of("project"));

        let dir = PathBuf::from(dir_string);

        let old_module = command.value_of("old").unwrap(); // okay because a subcommand is required

        let new_module = command.value_of("new").unwrap(); // okay beacause a subcommand is required

        let config_project = get_config(&dir, ".hs", ".cabal", command.is_present("copy"));

        if command.is_present("stash") {
            git_commit(&config_project.dir.to_string_lossy().to_string());
        }

        replace_all(&config_project, old_module, new_module);

        if command.is_present("spec") {
            let mut old_module_owned = old_module.to_string();
            let mut new_module_owned = new_module.to_string();
            old_module_owned.push_str("Spec");
            new_module_owned.push_str("Spec");
            replace_all(&config_project, &old_module_owned, &new_module_owned);
        }

    } else if let Some(command) = matches.subcommand_matches("idris") {

        let dir_string = get_dir(command.value_of("project"));

        let dir = PathBuf::from(dir_string);

        let old_module = command.value_of("old").unwrap(); // okay because a subcommand is required

        let new_module = command.value_of("new").unwrap(); // okay beacause a subcommand is required

        let config_project = get_config(&dir, ".idr", ".ipkg", command.is_present("copy"));

        if command.is_present("stash") {
            git_commit(&config_project.dir.to_string_lossy().to_string());
        }

        replace_all(&config_project, old_module, new_module);

    } else if let Some(command) = matches.subcommand_matches("elm") {

        let dir_string = get_dir(command.value_of("project"));

        let dir = PathBuf::from(dir_string);

        let old_module = command.value_of("old").unwrap(); // okay because a subcommand is required

        let new_module = command.value_of("new").unwrap(); // okay beacause a subcommand is required

        let config_project = get_config(&dir, ".elm", ".json", command.is_present("copy"));

        if command.is_present("stash") {
            git_commit(&config_project.dir.to_string_lossy().to_string());
        }

        replace_all(&config_project, old_module, new_module);

    } else {
        eprintln!("{}: failed to supply a subcommand", "Error".red());
    }
}
