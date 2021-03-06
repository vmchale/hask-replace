#![feature(test)]
#![allow(unused_imports)]

extern crate test;
extern crate cabal;
extern crate nom;

use nom::IResult;
use cabal::cabal::parse_all;
use cabal::hask::parse_full;
use test::test::Bencher;
use std::fs::File;
use std::io::prelude::*;

#[bench]
fn bench_hask(b: &mut Bencher) {
    let string = hask();
    b.iter(|| parse_full(&string, "Mod", "NewMod"))
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
    b.iter(|| concat_str(all(parse_full(&string, "Mod", "NewMod"))))
}

fn concat_str(xs: Vec<&str>) -> String {
    xs.into_iter().fold("".to_string(), |acc, x| acc + x)
}

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

fn cabal() -> String {
    read_file("data/cata/cata.cabal")
}

fn hask() -> String {
    read_file("data/cata/Mod.hs")
}
