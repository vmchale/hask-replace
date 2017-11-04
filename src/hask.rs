use nom::multispace;
use cabal::*;
use utils::*;
use nom::{rest_s, space};

pub fn parse_haskell(
    input: &str,
    file_type: &str,
    file_name: &str,
    old: &str,
    new: &str,
) -> String {
    concat_str(handle_errors(
        parse_full(input, old, new),
        file_type,
        file_name,
    ))
}

named!(skip<&str, Vec<&str>>,
  alt!(
    skip_comment |
    block_comment
  )
);

named!(block_comment<&str, Vec<&str>>,
  do_parse!(
    a: tag!("{-") >>
    b: take_until!("-}") >>
    c: tag!("-}") >>
    (vec![a, b, c])
  )
);

named_args!(pub parse_import_list<'a>(old: &'a str, new: &'a str)<&'a str, Vec<&'a str>>,
  do_parse!(
    ts: many0!(
      alt!(
        skip |
        boring_line
      )
    ) >>
    t: many0!(
      do_parse!(
        z: tag!("import") >> 
        b: opt!(multispace) >>
        c: many0!(skip) >>
        d : is_not!("( \n") >>
        (join(vec![vec![z, from_opt(b)], join(c), vec![swap_module(old, new, d)]]))
      )
    ) >>
    (join(vec![join(ts), join(t)]))
  )
);

named!(module<&str, Vec<&str>>,
  alt!(
    do_parse!(
      a: opt!(space) >>
      b: tag!("module ") >>
      (vec![from_opt(a), b])
    ) |
    do_parse!(
      a: opt!(space) >>
      b: tag!("signature ") >>
      (vec![from_opt(a), b])
    )
  )
);

named_args!(module_name<'a>(old: &'a str, new: &'a str)<&'a str, Vec<&'a str>>,
  do_parse!(
    a: module >>
    b: opt!(multispace) >>
    c: opt!(skip) >>
    e: is_not!("( \n") >>
    (join(vec![a, vec![from_opt(b)], from_vec(c), vec![swap_module(old, new, e)]]))
  )
);

named_args!(pub parse_full<'a>(old: &'a str, new: &'a str)<&'a str, Vec<&'a str>>,
  do_parse!(
    a: many0!(
      alt!(
        skip |
        boring_line
      )
    ) >>
    b: opt!(call!(module_name, old, new)) >>
    // e: opt!(is_not!("( \n")) >>
    f: opt!(call!(parse_import_list, old, new)) >>
    g: rest_s >>
    (join(vec![join(a), from_vec(b), from_vec(f), vec![g]]))
  )
);
