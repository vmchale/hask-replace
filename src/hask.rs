// use nom::multispace;
use cabal::*;
use utils::*;
use nom::space;

// opinionated find-and-replace
// we know already that monadic parser combinators work well.
// We want something like nom + loc

/// Perform find-and-replace on source file.
pub fn parse_haskell(
    input: &str,
    file_type: &str,
    file_name: &str,
    old: &str,
    new: &str,
) -> String {
    concat_str(handle_errors(
        parse_full(
            input,
            old,
            &(old.to_string() + "."),
            new,
            &(new.to_string() + "."),
        ),
        file_type,
        file_name,
    ))
}

// skip comment
named!(skip<&str, &str>,
  recognize!(alt!(
    skip_comment |
    block_comment
  ))
);

// skip block comment
named!(block_comment<&str, &str>,
  recognize!(do_parse!(
    a: tag!("{-") >>
    b: take_until!("-}") >>
    c: tag!("-}") >>
    ()
  ))
);

// parse import ... block
named!(pre_inputs<&str, ()>,
  do_parse!(
    tag!("import") >> 
    recognize!(opt!(multispace)) >>
    opt!(tag!("qualified ")) >>
    opt!(tag!("public ")) >>
    recognize!(many0!(skip)) >>
    (())
  )
);

named_args!(pub parse_import_list<'a>(old: &'a str, new: &'a str)<&'a str, Vec<&'a str>>,
  do_parse!(
    ts: recognize!(many0!(
      alt!(
        skip |
        boring_line
      )
    )) >>
    t: many0!(
      do_parse!(
        a: recognize!(pre_inputs) >>
        d: is_not!("( \n") >>
        f: take_until!("\n") >>
        g: is_a!("\n") >>
        (vec![a, swap_module(old, new, d), f, g])
      )
    ) >>
    (join(vec![vec![ts], join(t)]))
  )
);

// parse 'module' keyword
named!(pre_module<&str, ()>,
  do_parse!(
    a: opt!(space) >>
    b: tag!("module ") >>
    (())
  )
);

// parse 'signature' keyword
named!(pre_signature<&str, ()>,
  do_parse!(
    a: opt!(space) >>
    b: tag!("signature ") >>
    (())
  )
);

// parse 'module' or 'signature' keyword
named!(module<&str, &str>,
  recognize!(alt!(pre_module | pre_signature))
);

named!(pre_module_replace<&str, ()>,
  do_parse!(
    module >>
    opt!(space) >>
    many0!(skip) >>
    (())
  )
);

// replace module name
named_args!(module_name<'a>(old: &'a str, new: &'a str)<&'a str, Vec<&'a str>>,
  do_parse!(
    a: recognize!(pre_module_replace) >>
    e: is_not!("( \n") >>
    (vec![a, swap_module(old, new, e)])
  )
);

// parse a line, substituting when necessary.
named_args!(interesting_line<'a>(old: &'a str, old_dot: &'a str, new: &'a str, new_dot: &'a str)<&'a str, Vec<&'a str>>,
  many0!(
    alt!(
      do_parse!(a: tag!(old_dot) >> (swap_module(old_dot, new_dot, a))) |
      is_not!("ABCDEFGHIJKLMNOPQRSTUVWXYZ-{") |
      is_not!(" \n-{") |
      recognize!(
        alt!(
          do_parse!(tag!("-") >> is_not!("-ABCDEFGHIJKLMNOPQRSTUVWXYZ") >> (())) |
          do_parse!(tag!("{") >> is_not!("-ABCDEFGHIJKLMNOPQRSTUVWXYZ") >> (())) |
          do_parse!(tag!("-") >> is_not!("-") >> (())) |
          do_parse!(tag!("{") >> is_not!("-") >> (()))
        )
      )
    )
  )
);

// parse a line or skip commented line
named_args!(qualifier_substitution<'a>(old: &'a str, old_dot: &'a str, new: &'a str, new_dot: &'a str)<&'a str, Vec<&'a str>>,
  do_parse!(
    a: many0!(
      alt!(
        do_parse!(a: skip >> (vec![a])) |
        do_parse!(a: call!(interesting_line, old, old_dot, new, new_dot) >> (a))
      )
    ) >>
    (join(a))
  )
);

named_args!(pub parse_full<'a>(old: &'a str, old_dot: &'a str, new: &'a str, new_dot: &'a str)<&'a str, Vec<&'a str>>,
  do_parse!(
    a: recognize!(many0!(
      alt!(
        skip |
        boring_line
      )
    )) >>
    b: opt!(call!(module_name, old, new)) >>
    f: opt!(call!(parse_import_list, old, new)) >>
    g: call!(qualifier_substitution, old, old_dot, new, new_dot) >>
    (join(vec![vec![a], from_vec(b), from_vec(f), g]))
  )
);
