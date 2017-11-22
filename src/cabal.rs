extern crate colored;

use utils::*;
use nom::{rest_s, IResult, space, line_ending};
use std::process::exit;
use self::colored::*;

pub fn handle_errors<T>(input: IResult<&str, T, u32>, file_type: &str, file_name: &str) -> T {
    match input {
        IResult::Done(_, x) => x,
        _ => {
            eprintln!(
                "{}: Could not parse {} file at {}",
                "Error".red(),
                file_type,
                file_name,
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

// FIXME rewrite literally everything? lol.
named!(pub boring_line<&str, &str>,
  recognize!(do_parse!(
    a: alt!(is_a!(" ") | tag!("")) >>
    b: not!(alt!(tag!("module") | tag!("signature") | tag!("import") | tag!("name") | tag!("Name") | tag!("exposed-modules") | tag!("Exposed-modules") | tag!("Other-modules") | tag!("Exposed-Modules") | tag!("Other-Modules") | tag!("other-modules") | tag!("extra-source-files") | tag!("\"exposed-modules\":"))) >>
    c: take_until!("\n") >>
    d: tag!("\n") >>
    () // vec![a, b, c, d])
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
    v: many1!(do_parse!(a: step_indented >> b: is_not!("\n, ") >> (vec![a, swap_module(old_src, new_src, b)]))) >>
    (join(v))
  )
);

named_args!(parse_source<'a>(old_src: &'a str, new_src: &'a str)<&'a str, Vec<&'a str>>,
    do_parse!(
      a: recognize!(opt!(skip_stuff)) >>
      b: tag!("extra-source-files:") >>
      s: call!(parse_once, old_src, new_src) >>
      d: opt!(skip_stuff) >>
      (join(vec![join(vec![vec![a, b], s, from_vec(d)])])) // join(from_vec(d))]))
    )
);

// FIXME we should do replacements in the haddocks as well.
// links will be inside quotes "" while we can also have
// Module: ModuleName
// at the beginning of the haddocks.
named!(pub skip_comment<&str, &str>,
  recognize!(do_parse!(
    a: tag!("--") >>
    b: take_until!("\n") >>
    c: tag!("\n") >>
    () // vec![a, b, c])
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

named_args!(pub parse_all<'a>(old: &'a str, new: &'a str, old_src: &'a str, new_src: &'a str)<&'a str, Vec<&'a str>>,
  do_parse!(
    a: many1!(
        do_parse!(
        a: skip_stuff >>
        y: opt!(call!(parse_source, old_src, new_src)) >>
        w: opt!(space) >>
        z: alt!(tag!("other-modules:") | tag!("exposed-modules:") | tag!("name:") | tag!("Name:") | tag!("Exposed-modules:") | tag!("Other-modules:") | tag!("Exposed-Modules:") | tag!("Other-Modules:") | tag!("modules =") | tag!("\"exposed-modules\":")) >>
        b: call!(parse_modules, old, new) >>
        c: recognize!(opt!(skip_stuff)) >>
        (join(vec![vec![a, from_vec(y)], vec![vec![from_opt(w)], vec![z]], vec![b], vec![vec![c]]]))
        )
    ) >>
    b: rest_s >>
    (join(vec![join(join(a)), vec![b]]))
  )
);

named!(step_indented<&str, &str>,
  alt!(
    recognize!(do_parse!(a: tag!(",") >> b: multispace >> ())) |
    is_a!(" ") |
    recognize!(do_parse!(a: opt!(tag!("\n")) >> b: eof!() >> ())) | // (vec![from_opt(a), b])) |
    recognize!(do_parse!(c: opt!(tag!(",")) >> a: tag!("\n") >> b: multispace >> ())) // (vec![from_opt(c), a, b]))
  )
);

named_args!(module_helper<'a>(old: &'a str, new: &'a str)<&'a str, Vec<&'a str>>,
  do_parse!(z: opt!(skip_comment) >> a: step_indented >> x: opt!(tag!(", ")) >> b: is_not!("\r\n, ") >> c: alt!(tag!(",\n") | tag!(",") | line_ending ) >> (vec![from_opt(z), from_opt(x), a, swap_module(old, new, b), c]))
);

named_args!(parse_modules<'a>(old: &'a str, new: &'a str)<&'a str, Vec<&'a str>>,
  do_parse!(
    first: do_parse!(z: opt!(skip_comment) >> a: step_indented >> x: opt!(tag!(", ")) >> b: is_not!("\r\n, ") >> c: alt!(tag!(",\n") | tag!(",") | line_ending | eof!()) >> (vec![from_opt(z), from_opt(x), a, swap_module(old, new, b), c])) >>
    v: many0!(call!(module_helper, old, new)) >>
    (join(join(vec![vec![first], v])))
  )
);
