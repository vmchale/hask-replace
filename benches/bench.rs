#![feature(test)]
#![allow(unused_imports)]

extern crate test;
extern crate hreplace;
extern crate nom;

use nom::IResult;
use hreplace::cabal::parse_all;
use hreplace::hask::parse_full;
use test::test::Bencher;
use std::fs::File;
use std::io::prelude::*;

#[bench]
fn bench_hask(b: &mut Bencher) {
    let string = hask();
    b.iter(|| {
        parse_full(&string, "Mod", "Mod.", "NewMod", "NewMod.", "M-{")
    })
}

#[bench]
fn bench_cabal(b: &mut Bencher) {
    let string = cabal();
    b.iter(|| {
        parse_all(&string, "Mod", "NewMod", "stack.yaml", "bad.yaml")
    })
}
#[bench]
fn bench_hask_everything(b: &mut Bencher) {
    let string = hask();
    b.iter(|| {
        concat_str(all(parse_full(
            &string,
            "Mod",
            "Mod.",
            "NewMod",
            "NewMod.",
            "M-{",
        )))
    })
}

fn concat_str(xs: Vec<&str>) -> String {
    xs.into_iter().fold("".to_string(), |acc, x| acc + x)
}

fn all<T>(input: IResult<&str, T, u32>) -> T {
    match input {
        Ok((_, x)) => x,
        Err(e) => panic!("{:?}", e),
    }
}

fn read_file(file_name: &str) -> String {

    let mut file = File::open(file_name).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    contents

}

fn cabal() -> String {
    read_file("data/cata/cata.cabal")
}

fn hask() -> String {
    read_file("data/cata/Mod.hs")
}
