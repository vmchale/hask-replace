#[macro_use]
extern crate clap;
extern crate colored;
extern crate hreplace;
extern crate rayon;
extern crate walkdir;

use std::fs;
use rayon::prelude::*;
use clap::{App, AppSettings};
use std::path::PathBuf;
use walkdir::WalkDir;
use std::process::exit;
use colored::*;
use std::fs::{read_dir, remove_dir, File};
use std::io::prelude::*;
use std::process::Command;
use std::path::Path;
use std::fmt::Debug;
use hreplace::cabal::parse_cabal;
use hreplace::hask::parse_haskell;

#[derive(Debug)]
struct ProjectOwned {
    pub copy: bool,
    pub dir: PathBuf,
    pub parent_dir: bool,
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
        (!path.to_string_lossy().to_string().contains(".stack-work"))
            && (!path.to_string_lossy().to_string().contains("dist")) && {
            let p_str = path.to_string_lossy().to_string();
            (find.into_iter()
                .fold(false, |bool_accumulator, string_ext| {
                    bool_accumulator || p_str.ends_with(string_ext)
                }))
        }
    });

    let vec: Vec<PathBuf> = iter.map(|x| x.path().to_path_buf()).collect();

    vec
}

fn get_config(p: &PathBuf, module_ext: &[String], config_ext: &str, copy: bool) -> ProjectOwned {
    let s = p.to_string_lossy().to_string();
    let parent = if s.ends_with(".cabal") || s.ends_with(".ipkg") || s.ends_with(".json") {
        p.parent().unwrap_or(p)
    } else {
        p
    };

    let mut config_vec: Vec<String> = Vec::new();
    config_vec.push(config_ext.to_string());
    let vec = find_by_end_vec(p, config_vec.as_slice(), Some(2)); // FIXME only here cuz we don't do depths of > 2 lol.
    let vec_len = vec.len();

    // if we find more than one config file, abort.
    if vec_len > 1 && config_ext == ".config" {
        eprintln!(
            "{}: more than one '{}' file in indicated directory, aborting.",
            config_ext,
            "Error".red()
        );
        exit(0x0001)
    } else if vec_len > 1 && config_ext == ".cabal" {
        let config_name = vec.into_iter()
            .filter(|p| p.to_string_lossy().ends_with("Cabal.cabal"))
            .collect::<Vec<PathBuf>>()
            .pop()
            .unwrap();
        ProjectOwned {
            copy: copy,
            dir: PathBuf::from(s),
            parent_dir: true,
            config_file: { config_name },
            module_extension: module_ext.to_owned(),
            config_extension: config_ext.to_string(),
        }
    } else if vec_len == 0 {
        ProjectOwned {
            copy: copy,
            dir: parent.to_path_buf(),
            parent_dir: false,
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
            parent_dir: false,
            config_file: { config_name },
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

fn clean_empty_dirs(p: &PathBuf) -> () {
    let s = p.to_string_lossy().to_string();
    let dir = WalkDir::new(s).into_iter();

    let _ = dir.filter_map(|e| e.ok())
        .filter(|x| {
            x.file_type().is_dir()
                && read_dir(x.path())
                    .map(|inner| inner.into_iter().count())
                    .unwrap_or(0) == 0
        })
        .for_each(|p| {
            let intermediate = remove_dir(p.path());
            let _ = match intermediate {
                Ok(y) => y,
                Err(_) => eprintln!(
                    "{}: failed to clean up leftover directories.",
                    "Warning".yellow()
                ),
            };
        });
}

fn get_source_files(p: &PathBuf, extension: &[String]) -> Vec<PathBuf> {
    let s = p.to_string_lossy().to_string();

    let dir = if s == "" {
        WalkDir::new(".").into_iter()
    } else {
        WalkDir::new(s).into_iter()
    };

    let filtered = dir.filter_map(|e| e.ok()).filter(|p| {
        let path = p.path();
        (!path.to_string_lossy().to_string().contains(".stack-work"))
            && (!path.to_string_lossy().to_string().contains("dist")) && {
            let p_str = p.file_name().to_string_lossy().to_string();
            extension
                .into_iter()
                .fold(false, |bool_accumulator, string_ext| {
                    bool_accumulator || p_str.ends_with(string_ext)
                })
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
    // println!("{:?}", new_vec);
    new_vec
}

fn replace_file(
    p: &PathBuf,
    old_module: &str,
    new_module: &str,
    source_contents: &str,
    _: &[String],
    _: bool,
) -> () {
    let mut source_file = File::open(p).expect("174");
    let mut source = String::new();
    source_file.read_to_string(&mut source).expect("176");

    let replacements = parse_haskell(
        &source,
        source_contents,
        &p.to_string_lossy().to_string(),
        old_module,
        new_module,
    );

    write_file(p, &replacements);
}

fn rayon_directory_contents(
    config: &ProjectOwned,
    old_module: &str,
    new_module: &str,
    extension: &[String],
    source_contents: &str,
    _: bool,
) -> () {
    let dir: Vec<PathBuf> = if !config.parent_dir {
        get_source_files(&config.dir, extension)
    } else {
        let p = &config.dir;
        get_source_files(&p.parent().unwrap_or(p).to_path_buf(), extension)
    };
    let iter = dir.into_par_iter();

    iter.for_each(|p| {
        let mut source_file = File::open(&p).unwrap();
        let mut source = String::new();
        source_file.read_to_string(&mut source).unwrap();
        let replacements = parse_haskell(
            &source,
            source_contents,
            &p.to_string_lossy().to_string(),
            old_module,
            new_module,
        );
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
            eprintln!("{}: Failed to read file at: {:?}", "Error".red(), p);
            exit(0x0001)
        }
    }

    contents
}

fn trim_matches_only<'a>(name_str: &'a mut str, old_str: &'a [String]) -> (String, String) {
    let mut default = "".to_string();
    let mut default_name = "".to_string();
    for p in old_str {
        if name_str.ends_with(p) {
            default = p.to_string();
            default_name = name_str.trim_right_matches(p).to_string();
        }
    }
    if default == "".to_string() && default_name == "".to_string() {
        for p in old_str {
            if name_str.replace(".hs", ".y").ends_with(p) {
                default = p.replace(".hs", ".y").to_string();
                default_name = name_str
                    .replace(".hs", ".y")
                    .trim_right_matches(p)
                    .to_string();
            }
        }
    }
    let extn = default.chars().skip_while(|c| c != &'.').collect();
    (default_name, extn)
}

fn replace_all(
    config: &ProjectOwned,
    old_module: &str,
    new_module: &str,
    source_contents: &str,
    benchmark_mode: bool,
) -> () {
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
        // FIXME pop of multiple values?
        let name = old_module_vec.pop().unwrap();
        let name_string: String = name.to_string_lossy().to_string();
        let name_str: &mut str = &mut name_string.as_str().to_owned();
        let old_string: Vec<String> = module_to_file_names(old_module, module_ext);
        let old_str: &[String] = &old_string;
        let (dir, extn) = trim_matches_only(name_str, old_str);
        (name, dir.to_string(), extn)
    } else {
        eprintln!(
            "{}: module '{}' does not exist in this project",
            "Error".red(),
            old_module
        );
        exit(0x0001)
    };

    let config_string = config.get_config_path(module_ext, config_ext);

    let contents = read_file(&config_string);

    let in_config_file = (&contents).contains(old_module);

    // TODO purescript thing doesn't actually get moved??
    if !in_config_file && !config.config_extension.ends_with(".json")
        && !config.config_extension.ends_with(".yaml")
    {
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
    let replacements = parse_cabal(
        &source,
        config_ext,
        &config_string,
        old_module,
        new_module,
        None,
    );

    write_file(&config_string, &replacements);

    // step 4: replace every 'import Module' with 'import NewModule'
    if !config.copy {
        rayon_directory_contents(
            config,
            old_module,
            new_module,
            module_ext,
            source_contents,
            false,
        );
    }

    // step 5: move the actual file
    let mut new_module_path = src_dir;
    new_module_path.push_str(&module_to_file_names(new_module, module_ext)
        .into_iter()
        .filter(|p| p.ends_with(&extn))
        .next()
        .unwrap());
    if Path::new(&new_module_path).exists() && !benchmark_mode {
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

        replace_file(
            &file_name,
            old_module,
            new_module,
            source_contents,
            module_ext,
            false,
        );
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

    // step 6: clean up any now-spurious directories
    clean_empty_dirs(&config.dir);
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
    if let Some(x) = matches.subcommand_matches("update") {
        let force = x.is_present("force");

        println!("current version: {}", crate_version!());

        let s = if force {
            "curl -LSfs https://japaric.github.io/trust/install.sh | sh -s -- --git vmchale/hask-replace --force"
        } else {
            "curl -LSfs https://japaric.github.io/trust/install.sh | sh -s -- --git vmchale/hask-replace"
        };

        let script = Command::new("bash")
            .arg("-c")
            .arg(s)
            .output()
            .expect("failed to execute update script.");

        let script_string = String::from_utf8(script.stderr).unwrap();

        println!("{}", script_string);
    } else if let Some(command) = matches.subcommand_matches("module") {
        let dir_string = get_dir(command.value_of("project"));

        let dir = PathBuf::from(dir_string);

        let old_module = command.value_of("old").unwrap();

        let new_module = command.value_of("new").unwrap();

        // TODO .hs-boot, .hsig files (?)
        let extns = vec![".hs".to_string(), ".x".to_string(), ".y".to_string()];

        let config_extn = if command.is_present("hpack") {
            "package.yaml"
        } else {
            ".cabal"
        };
        let config_project = get_config(&dir, &extns, config_extn, command.is_present("copy"));

        if command.is_present("stash") {
            git_stash(&config_project.dir.to_string_lossy().to_string());
        }

        let benchmark = command.is_present("bench");

        replace_all(
            &config_project,
            old_module,
            new_module,
            "Haskell",
            benchmark,
        );

        if command.is_present("spec") {
            let mut old_module_owned = old_module.to_string();
            let mut new_module_owned = new_module.to_string();
            old_module_owned.push_str("Spec");
            new_module_owned.push_str("Spec");
            replace_all(
                &config_project,
                &old_module_owned,
                &new_module_owned,
                "Haskell",
                benchmark,
            );
        }
    } else if let Some(command) = matches.subcommand_matches("rename") {
        let dir_string = get_dir(command.value_of("project"));

        let dir = PathBuf::from(dir_string);

        let old_config = command.value_of("old").unwrap();

        let new_config = command.value_of("new").unwrap();

        let extns = vec![".cabal".to_string()];

        let config_project = get_config(&dir, &extns, ".project", false); // TODO .local too?

        if command.is_present("stash") {
            git_stash(&config_project.dir.to_string_lossy().to_string());
        }

        replace_all(&config_project, old_config, new_config, "Cabal", false);
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

        replace_all(&config_project, old_module, new_module, "Idris", false);
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

        replace_all(&config_project, old_module, new_module, "Elm", false);
    } else if let Some(command) = matches.subcommand_matches("purescript") {
        let dir_string = get_dir(command.value_of("project"));

        let dir = PathBuf::from(dir_string);

        let old_module = command.value_of("old").unwrap();

        let new_module = command.value_of("new").unwrap();

        let extns = vec![".purs".to_string()];

        let config_project = get_config(&dir, &extns, ".json", command.is_present("copy"));

        if command.is_present("stash") {
            git_stash(&config_project.dir.to_string_lossy().to_string());
        }

        replace_all(&config_project, old_module, new_module, "PureScript", false);
    } else {
        eprintln!("{}: failed to supply a subcommand", "Error".red());
    }
}
