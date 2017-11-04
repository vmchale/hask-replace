use nom::multispace;
use cabal::*;
use utils::*;
use nom::rest_s;

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
        a: take_until!("import") >>
        z: tag!("import") >> 
        b: opt!(multispace) >>
        c: many0!(skip) >>
        d : is_not!("( \n") >>
        (join(vec![vec![a, z, from_opt(b)], join(c), vec![swap_module(old, new, d)]]))
      )
    ) >>
    (join(vec![join(ts), join(t)]))
  )
);

named!(module<&str, Vec<&str>>,
  alt!(
    do_parse!(
      a: take_until!("module") >>
      b: tag!("module ") >>
      (vec![a, b])
    ) |
    do_parse!(
      a: take_until!("signature") >>
      b: tag!("signature ") >>
      (vec![a, b])
    )
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
    b: module >>
    c: opt!(multispace) >>
    d: opt!(skip) >>
    e: is_not!("( ") >>
    f: call!(parse_import_list, old, new) >>
    g: rest_s >>
    (join(vec![join(a), b, vec![from_opt(c)], from_vec(d), vec![swap_module(old, new, e)], f, vec![g]]))
  )
);
