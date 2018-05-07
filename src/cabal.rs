extern crate colored;
extern crate smallvec;

use utils::*;
use nom::{line_ending, rest_s, space, IResult};
use std::process::exit;
use self::colored::*;
use self::smallvec::SmallVec;
use std::path::Path;

pub struct Version {
    pub version: SmallVec<[u16; 4]>,
}

pub struct PackageIdentifier<'a> {
    pub pkg_name: &'a str,
    pub pkg_version: Version,
}

pub enum License<'a> {
    GPL { version: Option<Version> },
    UnknownLicense { name: Option<&'a str> },
}

pub struct PackageDescription<'a> {
    pub pkg_identifier: PackageIdentifier<'a>,
    pub name: Option<&'a str>,
    pub version: Option<Version>,
    pub license: Option<&'a str>,
    pub license_files: Vec<&'a Path>,
}

pub fn handle_errors<T>(input: IResult<&str, T, u32>, file_type: &str, file_name: &str) -> T {
    match input {
        Ok((_, x)) => x, // IResult::Done(_, x) => x,
        Err(e) => {
            eprintln!(
                "{}: Could not parse {} file at {}\n{:?}",
                "Error".red(),
                file_type,
                file_name,
                e,
            );
            exit(0x001)
        } // FIXME
    }
}

pub fn parse_cabal(
    input: &str,
    file_type: &str,
    file_name: &str,
    old: &str,
    new: &str,
    src: Option<(&str, &str)>,
) -> String {
    match src {
        Some((old_src, new_src)) => concat_str(handle_errors(
            parse_all(input, old, new, old_src, new_src),
            file_type,
            file_name,
        )),
        _ => concat_str(handle_errors(
            parse_all(input, old, new, "", ""),
            file_type,
            file_name,
        )),
    }
}

named!(pub boring_line<&str, &str>,
  recognize!(do_parse!(
    a: opt!(multispace) >>
    b: not!(
      alt!(
        tag!("module") |
        tag!("signature") |
        tag!("import") |
        recognize!(do_parse!(is_a!(",) ") >> tag!("module") >> (()))) |
        tag!("exposed-modules") |
        tag!("Exposed-modules") |
        tag!("Other-modules") |
        tag!("Exposed-Modules") |
        tag!("Other-Modules") |
        tag!("other-modules") |
        tag!("packages") |
        tag!("extra-source-files") |
        tag!("\"exposed-modules\":") |
        tag!("\"depends\":") |
        tag!("\"dependencies\":")
      )) >>
    c: take_until!("\n") >>
    d: tag!("\n") >>
    ()
  ))
);

named!(pub parse_packages<&str, Vec<&str>>,
  do_parse!(
    a: take_until!("packages:") >>
    b: tag!("packages:") >>
    c: call!(parse_once, "", "") >>
    (join(vec![vec![a, b], c]))
  )
);

named_args!(parse_once<'a>(old_src: &'a str, new_src: &'a str)<&'a str, Vec<&'a str>>,
  do_parse!(
    v: many1!(
      do_parse!(
        a: step_indented >>
        b: is_not!("\n, ") >>
        (vec![a, swap_module(old_src, new_src, b)])
      )
    ) >>
    (join(v))
  )
);

// jump to relevant stuff.
named!(cabal_head<&str, &str>,
  recognize!(do_parse!(
    a: recognize!(opt!(skip_stuff)) >>
    b: tag!("extra-source-files:") >>
    (())
  ))
);

named_args!(parse_source<'a>(old_src: &'a str, new_src: &'a str)<&'a str, Vec<&'a str>>,
    do_parse!(
      a: cabal_head >>
      s: call!(parse_once, old_src, new_src) >>
      d: recognize!(opt!(skip_stuff)) >>
      (join(vec![vec![a], s, vec![d]]))
    )
);

named!(pub skip_comment<&str, &str>,
  recognize!(do_parse!(
    a: tag!("--") >>
    b: take_until!("\n") >>
    c: tag!("\n") >>
    ()
  ))
);

named!(skip_stuff<&str, Vec<&str>>,
  many0!(
    alt!(
      skip_comment |
      boring_line
    )
  )
);

named!(pub multispace<&str, &str>,
  alt!(
    is_a!(" ") |
    tag!("")
  )
);

named!(prolegomena<&str, ()>,
  do_parse!(
    skip_stuff >>
    opt!(space) >>
    alt!(
      tag!("other-modules:") |
      tag!("exposed-modules:") |
      tag!("name:") |
      tag!("Name:") |
      tag!("Exposed-modules:") |
      tag!("Other-modules:") | 
      tag!("Exposed-Modules:") |
      tag!("Other-Modules:") |
      tag!("modules =") |
      tag!("packages:") |
      tag!("\"exposed-modules\":") | // FIXME what if exposed-modules doesn't exist?
      tag!("\"depends\":") |
      tag!("\"dependencies\":")
    ) >>
    (())
  )
);

named_args!(pub parse_all<'a>(old: &'a str, new: &'a str, old_src: &'a str, new_src: &'a str)<&'a str, Vec<&'a str>>,
  do_parse!(
    a: many1!(
        do_parse!(
          a: recognize!(skip_stuff) >>
          b: opt!(call!(parse_source, old_src, new_src)) >>
          d: recognize!(prolegomena) >>
          e: opt!(call!(parse_modules, old, new)) >> // hpack doesn't require an exposed-modules section lol ??
          c: recognize!(opt!(skip_stuff)) >>
          (join(vec![vec![a], from_vec(b), vec![d], from_vec(e), vec![c]]))
        )
    ) >>
    b: rest_s >>
    (join(vec![join(a), vec![b]]))
  )
);

named!(step_indented<&str, &str>,
  alt!(
    recognize!(do_parse!(a: tag!(",") >> b: multispace >> ())) |
    is_a!(" ") |
    recognize!(do_parse!(a: opt!(tag!("\n")) >> b: eof!() >> ())) |
    recognize!(do_parse!(c: opt!(tag!(",")) >> a: tag!("\n") >> b: multispace >> ()))
  )
);

named!(module_prolegomena<&str, ()>,
  do_parse!(
    opt!(skip_comment) >>
    step_indented >>
    opt!(tag!(", ")) >>
    opt!(tag!("- ")) >> // FIXME edit this.
    (())
  )
);

named_args!(module_helper<'a>(old: &'a str, new: &'a str)<&'a str, Vec<&'a str>>,
  do_parse!(a: recognize!(module_prolegomena) >> b: is_not!("\r\n, ") >> c: alt!(tag!(",\n") | tag!(",") | line_ending) >> (vec![a, swap_module(old, new, b), c]))
);

named_args!(parse_modules<'a>(old: &'a str, new: &'a str)<&'a str, Vec<&'a str>>,
  do_parse!(
    first: do_parse!(a: recognize!(module_prolegomena) >> b: is_not!("\r\n, ") >> c: alt!(tag!(",\n") | tag!(",") | line_ending | eof!()) >> (vec![a, swap_module(old, new, b), c])) >>
    v: many0!(call!(module_helper, old, new)) >>
    (join(join(vec![vec![first], v])))
  )
);
