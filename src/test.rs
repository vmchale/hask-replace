#![allow(unused_imports)]
#![allow(dead_code)]
#[cfg(test)]

use cabal::*;
use hask::*;
use std::fs::File;
use std::io::prelude::*;
use nom::IResult;
use utils::*;

fn all<T>(input: IResult<&str, T, u32>) -> T {
    match input {
        IResult::Done(_, x) => x,
        IResult::Error(e) => panic!("{}", e),
        IResult::Incomplete(x) => panic!("{:?}", x),
    }
}

fn read_file(file_name: &str) -> String {

    let mut file = File::open(file_name).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    contents

}

fn sig() -> String {
    read_file("data/sum/Sig.hsig")
}

fn hask() -> String {
    read_file("data/cata/Mod.hs")
}

fn modified_hask() -> String {
    read_file("data/cata/Mod.out")
}

fn modified_cabal() -> String {
    read_file("data/cata/cata.out")
}

fn project() -> String {
    read_file("data/cabal.project")
}

fn cabal() -> String {
    read_file("data/cata/cata.cabal")
}

#[test]
fn test_hask() {
    assert_eq!(
        concat_str(all(parse_full(
            &concat_str(all(parse_full(&hask(), "Mod", "NewMod"))),
            "Data.Functor.Foldable",
            "BadModuleName",
        ))),
        modified_hask()
    );
}

#[test]
fn test_extra_project() {
    let expected = "cata";
    assert_eq!(
        all(parse_packages(&project())).into_iter().last(),
        Some(expected)
    );
}

#[test]
fn test_exposed_modules() {
    assert_eq!(
        concat_str(all(parse_all(
            &cabal(),
            "Mod",
            "ReplacementMod",
            "stack.yaml",
            "bad.yaml",
        ))),
        modified_cabal()
    );
}

#[test]
fn test_signature_names() {
    let expected = "signature NewSig ( function ) where\n\nfunction :: (Num a) => [a] -> a\n"
        .to_string();
    assert_eq!(
        concat_str(all(parse_full(&sig(), "Sig", "NewSig"))),
        expected
    );
}
