#[macro_use]
extern crate clap;
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
    pub module_extension: Vec<String>,
    pub config_extension: String,
}

impl ProjectOwned {
    fn get_config_path(&self, module_ext: &[String], config_ext: &str) -> String {
        get_config(&self.dir, module_ext, config_ext, self.copy)
            .config_file
            .to_string_lossy()
            .to_string()
    }
}


fn find_by_end_vec(p: &PathBuf, find: &[String], depth: Option<usize>) -> Vec<PathBuf> {

    let s = p.to_string_lossy().to_string();

    let dir = if let Some(d) = depth {
        WalkDir::new(&s).max_depth(d)
    } else {
        WalkDir::new(&s)
    };
    let iter = dir.into_iter().filter_map(|e| e.ok()).filter(|p| {
        let path = p.path();
        (!path.starts_with(".stack-work")) &&
            {
                let p_str = path.to_string_lossy().to_string();
                (find.into_iter().fold(
                    false,
                    |bool_accumulator, string_ext| {
                        bool_accumulator || p_str.ends_with(string_ext)
                    },
                ))
            }
    });

    let vec: Vec<PathBuf> = iter.map(|x| x.path().to_path_buf()).collect();

    vec
}


fn get_config(p: &PathBuf, module_ext: &[String], config_ext: &str, copy: bool) -> ProjectOwned {

    let parent = p.parent().unwrap_or(p);
    let s = p.to_string_lossy().to_string();

    let mut config_vec: Vec<String> = Vec::new();
    config_vec.push(config_ext.to_string());
    let vec = find_by_end_vec(p, config_vec.as_slice(), Some(1));
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
            module_extension: module_ext.to_owned(),
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
            module_extension: module_ext.to_owned(),
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

fn get_source_files(p: &PathBuf, extension: &[String]) -> Vec<PathBuf> {

    let s = p.to_string_lossy().to_string();

    let dir = WalkDir::new(s).into_iter();

    let filtered = dir.filter_map(|e| e.ok()).filter(|p| {
        let path = p.path();
        !path.starts_with(".stack-work") &&
            {
                let p_str = p.file_name().to_string_lossy().to_string();
                extension.into_iter().fold(
                    false,
                    |bool_accumulator, string_ext| {
                        bool_accumulator || p_str.ends_with(string_ext)
                    },
                )
            }
    });

    filtered.map(|p| p.path().to_path_buf()).collect()

}

fn module_to_file_names(module: &str, extension: &[String]) -> Vec<String> {
    let replacements = module.replace(".", "/");
    let mut replacements_vec: Vec<String> = Vec::with_capacity(extension.into_iter().count());
    for _ in extension {
        replacements_vec.push(replacements.clone());
    }
    let mut extension_iter = extension.into_iter();
    let new_vec: Vec<String> = replacements_vec
        .into_iter()
        .map(|mut x| {
            x.push_str(extension_iter.next().unwrap());
            x
        })
        .collect::<Vec<String>>();
    new_vec
}

fn replace_file(
    p: &PathBuf,
    old_module: &str,
    new_module: &str,
    extension: &[String],
    whole_directory: bool,
) -> () {

    let mut source_file = File::open(p).expect("139");
    let mut source = String::new();
    source_file.read_to_string(&mut source).expect("141");
    let mut old_module_regex = old_module.to_string();
    if !whole_directory {
        old_module_regex.push_str("(\n|\\.[a-z]|( +)as|( +)exposing.*\n|( +)\\(|( +)where)+?");
    } else {
        old_module_regex.push_str(
            "(\n|\\.[a-zA-Z]|( +)as|( +)exposing.*\n|( +)\\(|( +)where)+?",
        );
    }
    let re = Regex::new(&old_module_regex).unwrap();
    let num = if extension.into_iter().next().unwrap() == ".idr" {
        1
    } else {
        0
    }; // FIXME
    let replacements = re.replacen(&source, num, |caps: &Captures| {
        format!("{}{}", new_module, &caps[1])
    }).to_string();
    write_file(p, &replacements);

}

fn rayon_directory_contents(
    config: &ProjectOwned,
    old_module: &str,
    new_module: &str,
    extension: &[String],
    whole_directory: bool,
) -> () {

    let dir: Vec<PathBuf> = get_source_files(&config.dir, extension);
    let iter = dir.into_par_iter();
    let mut old_module_regex = old_module.to_string();
    if !whole_directory {
        old_module_regex.push_str("(\n|\\.[a-z]|( +)as|( +)exposing.*\n|( +)\\(|( +)where)+?");
    } else {
        old_module_regex.push_str(
            "(\n|\\.[a-zA-Z]||( +)as|( +)exposing.*\n|( +)\\(|( +)where)+?",
        );
    }
    let re = Regex::new(&old_module_regex).unwrap();

    iter.for_each(|p| {
        let mut source_file = File::open(&p).unwrap();
        let mut source = String::new();
        source_file.read_to_string(&mut source).unwrap();
        let num = if extension.into_iter().next().unwrap() == ".idr" {
            1
        } else {
            0
        }; // FIXME
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

fn trim_matches_only<'a>(name_str: &'a mut str, old_str: &'a [String]) -> (&'a str, String) {
    let mut default = "".to_string();
    let mut default_name = "";
    for p in old_str {
        if name_str.ends_with(p) {
            default = p.to_string();
            default_name = name_str.trim_right_matches(p);
        }
    }
    let extn = default.chars().skip_while(|c| c != &'.').collect();
    (default_name, extn)
}


fn replace_all(config: &ProjectOwned, old_module: &str, new_module: &str) -> () {

    let module_ext: &Vec<String> = &config.module_extension;
    let config_ext: &str = &config.config_extension;

    // step 1: determine that the module we want to replace in fact exists
    let mut old_module_vec = find_by_end_vec(
        &config.dir,
        &module_to_file_names(old_module, module_ext),
        None,
    );
    let old_module_exists = !(old_module_vec.is_empty());

    let (old_module_name, src_dir, extn) = if old_module_exists {
        // TODO these should return multiple possible values.
        let name = old_module_vec.pop().unwrap();
        let name_string: String = name.to_string_lossy().to_string();
        let name_str: &mut str = &mut name_string.as_str().to_owned();
        let old_string: Vec<String> = module_to_file_names(old_module, module_ext);
        let old_str: &[String] = &old_string; // old_string.as_str();
        let (dir, extn) = trim_matches_only(name_str, old_str);
        (name, dir.to_string(), extn)
    } else {
        eprintln!("module '{}' does not exist in this project", old_module);
        exit(0x0001)
    };

    let config_string = config.get_config_path(module_ext, config_ext);

    let contents = read_file(&config_string);

    let in_config_file = (&contents).contains(old_module);

    if !in_config_file && config.config_extension != ".json" {
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
    let mut old_module_regex = "".to_string();
    old_module_regex.push_str(old_module);
    old_module_regex.push_str("(\\)|\n|,)+?");
    let re = Regex::new(&old_module_regex).unwrap();
    let replacements = if !config.copy {
        re.replacen(&source, 0, |caps: &Captures| {
            format!("{}{}", new_module, &caps[1])
        }).to_string()
    } else {
        re.replacen(&source, 0, |caps: &Captures| {
            let filtered_caps: String = (&caps[1])
                .to_string()
                .chars()
                .filter(|c| c != &'\n')
                .collect();
            format!(
                "{}{}, {}{}",
                new_module,
                &filtered_caps,
                old_module,
                &caps[1]
            ) // FIXME this should be handled differently! This is bad.
        }).to_string()
    };

    write_file(&config_string, &replacements);

    // step 4: replace every 'import Module' with 'import NewModule'
    if !config.copy {
        rayon_directory_contents(config, old_module, new_module, module_ext, false);
    }

    // step 5: move the actual file
    let mut new_module_path = src_dir;
    new_module_path.push_str(&module_to_file_names(new_module, module_ext)
        .into_iter()
        .filter(|p| p.ends_with(&extn))
        .next()
        .unwrap());
    if Path::new(&new_module_path).exists() {
        eprintln!("{}: destination module already exists.", "Error".red());
        exit(0x0001);
    }

    let expr = if !config.copy {
        fs::rename(&old_module_name, &new_module_path)
    } else {
        fs::copy(&old_module_name, &new_module_path).map(|_| ())
    };

    if config.copy {

        let mut file_name: PathBuf = (&config).dir.to_owned();
        // FIXME don't hard-code this.
        file_name.push("src/");
        file_name.push(
            module_to_file_names(new_module, module_ext)
                .into_iter()
                .next()
                .unwrap(),
        );
        replace_file(&file_name, old_module, new_module, module_ext, false);

    }

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

fn git_stash(src_dir: &str) -> () {
    let mut cmd = "cd ".to_string();
    cmd.push_str(src_dir);
    cmd.push_str("&&");
    cmd.push_str("git stash -k");
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

fn main() {
    let yaml = load_yaml!("options-en.yml");
    let matches = App::from_yaml(yaml)
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequired)
        .get_matches();

    if let Some(command) = matches.subcommand_matches("module") {

        let dir_string = get_dir(command.value_of("project"));

        let dir = PathBuf::from(dir_string);

        let old_module = command.value_of("old").unwrap();

        let new_module = command.value_of("new").unwrap();

        let extns = vec![
            ".hs".to_string(),
            // ".hs-boot".to_string(),
            // ".hsig".to_string(),
        ];

        let config_project = get_config(&dir, &extns, ".cabal", command.is_present("copy"));

        if command.is_present("stash") {
            git_stash(&config_project.dir.to_string_lossy().to_string());
        }

        replace_all(&config_project, old_module, new_module);

        if command.is_present("spec") {
            let mut old_module_owned = old_module.to_string();
            let mut new_module_owned = new_module.to_string();
            old_module_owned.push_str("Spec");
            new_module_owned.push_str("Spec");
            replace_all(&config_project, &old_module_owned, &new_module_owned);
        }

    } else if let Some(command) = matches.subcommand_matches("rename") {

        let dir_string = get_dir(command.value_of("project"));

        let dir = PathBuf::from(dir_string);

        let old_config = command.value_of("old").unwrap();

        let new_config = command.value_of("new").unwrap();

        let extns = vec![".cabal".to_string(), ".yaml".to_string()];

        let config_project = if command.is_present("stack") {
            get_config(&dir, &extns, ".yaml", false)
        } else {
            get_config(&dir, &extns, ".cabal", false)
        };

        if command.is_present("stash") {
            git_stash(&config_project.dir.to_string_lossy().to_string());
        }

        replace_all(&config_project, old_config, new_config);

    } else if let Some(command) = matches.subcommand_matches("idris") {

        let dir_string = get_dir(command.value_of("project"));

        let dir = PathBuf::from(dir_string);

        let old_module = command.value_of("old").unwrap();

        let new_module = command.value_of("new").unwrap();

        let extns = vec![".idr".to_string()];

        let config_project = get_config(&dir, &extns, ".ipkg", command.is_present("copy"));

        if command.is_present("stash") {
            git_stash(&config_project.dir.to_string_lossy().to_string());
        }

        replace_all(&config_project, old_module, new_module);

    } else if let Some(command) = matches.subcommand_matches("elm") {

        let dir_string = get_dir(command.value_of("project"));

        let dir = PathBuf::from(dir_string);

        let old_module = command.value_of("old").unwrap();

        let new_module = command.value_of("new").unwrap();

        let extns = vec![".elm".to_string()];

        let config_project = get_config(&dir, &extns, ".json", command.is_present("copy"));

        if command.is_present("stash") {
            git_stash(&config_project.dir.to_string_lossy().to_string());
        }

        replace_all(&config_project, old_module, new_module);

    } else {
        eprintln!("{}: failed to supply a subcommand", "Error".red());
    }
}
