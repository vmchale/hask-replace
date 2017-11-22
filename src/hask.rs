// use nom::multispace;
use cabal::*;
use utils::*;
use nom::space;

// opinionated find-and-replace
// we know already that monadic parser combinators work well.
// We want something like nom + loc

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
        b: opt!(space) >>
        e: opt!(tag!("qualified ")) >>
        h: opt!(tag!("public ")) >>
        c: many0!(skip) >>
        d: is_not!("( \n") >>
        f: take_until!("\n") >>
        g: many1!(tag!("\n")) >>
        (join(vec![vec![z, from_opt(b)], join(c), vec![from_opt(e), from_opt(h), swap_module(old, new, d), f], g]))
      )
    ) >>
    (join(vec![join(ts), join(t)]))
  )
);

named!(pre_module<&str, Vec<&str>>,
  do_parse!(
    a: opt!(space) >>
    b: tag!("module ") >>
    (vec![from_opt(a), b])
  )
);

named!(pre_signature<&str, Vec<&str>>,
  do_parse!(
    a: opt!(space) >>
    b: tag!("signature ") >>
    (vec![from_opt(a), b])
  )
);

named!(module<&str, Vec<&str>>,
  alt!(pre_module | pre_signature)
);

named_args!(module_name<'a>(old: &'a str, new: &'a str)<&'a str, Vec<&'a str>>,
  do_parse!(
    a: module >>
    b: opt!(space) >>
    c: many0!(skip) >>
    e: is_not!("( \n") >>
    (join(vec![a, vec![from_opt(b)], join(c), vec![swap_module(old, new, e)]]))
  )
);

named_args!(interesting_line<'a>(old: &'a str, new: &'a str)<&'a str, Vec<Vec<&'a str>>>,
  many0!(
    alt!(
      do_parse!(a: tag!(old) >> b: tag!(".") >> (vec![new, b])) |
      do_parse!(a: is_not!(" ") >> (vec![a])) |
      do_parse!(a: space >> (vec![a]))
    )
  )
);

named_args!(qualifier_substitution<'a>(old: &'a str, new: &'a str)<&'a str, Vec<&'a str>>,
  do_parse!(
    a: many0!(
      alt!(
        do_parse!(a: skip >> (vec![a])) |
        call!(interesting_line, old, new)
      )
    ) >>
    (join(join(a)))
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
    f: opt!(call!(parse_import_list, old, new)) >>
    g: call!(qualifier_substitution, old, new) >>
    (join(vec![join(a), from_vec(b), from_vec(f), g]))
  )
);
