// use nom::multispace;
use cabal::*;
use utils::*;
use nom::{space, hex_digit};

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

    // this is less stupid than it looks because nom parses by byte.
    let special = ("-{\"'".to_string() + &old[0..1]).to_string();

    concat_str(handle_errors(
        parse_full(
            input,
            old,
            &(old.to_string() + "."),
            new,
            &(new.to_string() + "."),
            &special,
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
    many0!(skip) >>
    (())
  )
);

named!(pre_module_exports<&str, ()>,
  do_parse!(
    is_a!(" ,(") >>
    tag!("module") >>
    multispace >>
    many0!(skip) >>
    (())
  )
);

named_args!(parse_import_list<'a>(old: &'a str, new: &'a str)<&'a str, Vec<&'a str>>,
  do_parse!(
    ts: recognize!(many0!(
      alt!(
        skip |
        boring_line
      )
    )) >>
    t2: many0!(
      do_parse!(
        a: recognize!(pre_module_exports) >>
        d: is_not!(", \n)") >>
        f: recognize!(do_parse!(take_until!("\n") >> tag!("\n") >> (()))) >>
        (vec![a, swap_module(old, new, d), f])
      )
    ) >>
    ts2: recognize!(many0!(
      alt!(
        skip |
        boring_line
      )
    )) >>
    t: many0!(
      do_parse!(
        // a: alt!(recognize!(pre_inputs) | skip | tag!("\n")) >>
        a: recognize!(pre_inputs) >>
        d: is_not!("( \n") >>
        f: recognize!(do_parse!(take_until!("\n") >> is_a!("\n") >> (()))) >>
        (vec![a, swap_module(old, new, d), f])
      )
    ) >>
    (join(vec![vec![ts], join(t2), vec![ts2], join(t)]))
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
    z: recognize!(many0!(
      alt!(
        skip |
        boring_line
      )
    )) >>
    a: recognize!(pre_module_replace) >>
    e: is_not!(" \n(") >>
    (vec![z, a, swap_module(old, new, e)])
  )
);

named!(fancy_stuff<&str, &str>,
  recognize!(
    alt!(
      do_parse!(
        tag!("-") >> 
        is_not!("-") >>
        (())
      ) |
      do_parse!(
        tag!("{") >> 
        is_not!("-") >>
        (())
      )
    )
  )
);

// parse a line, substituting when necessary.
named_args!(interesting_line<'a>(old: &'a str, old_dot: &'a str, new: &'a str, new_dot: &'a str, special: &'a str)<&'a str, Vec<&'a str>>,
  many0!(
    alt_complete!(
      do_parse!(a: tag!(old_dot) >> (swap_module(old_dot, new_dot, a))) |
      skip |
      is_not!(special) |
      is_not!(" \n-{\"'") |
      recognize!(complete!(do_parse!(alt!(tag!("' ") | tag!("'\n") | tag!("']") | tag!("'t") | tag!("''") | tag!("'''") | tag!("')") | tag!("',")) >> (())))) |
      char_contents |
      string_contents |
      fancy_stuff
    )
  )
);

named!(take_unicode<&str, &str>,
  recognize!(do_parse!(
    tag!("\\") >>
    opt!(tag!("x")) >> // FIXME make it only do hex when 
    hex_digit >>
    (())
  ))
);

named!(linebreak_string<&str, &str>,
  recognize!(do_parse!(
    tag!("\\\n") >>
    multispace >>
    tag!("\\") >>
    (())
  ))
);

named!(char_contents<&str, &str>,
  recognize!(do_parse!(
    tag!("'") >>
    b: many1!(alt!(is_not!("\\'") | tag!("\\\\") | tag!("\\b") | tag!("\\f") | tag!("\\r") | tag!("\\t") | take_unicode | tag!("\\DEL") | tag!("\\NUL") | tag!("\\^M") | tag!("\\n"))) >>
    tag!("'") >>
    opt!(tag!("\n")) >>
    (())
  ))
);

named!(string_contents<&str, &str>,
  recognize!(do_parse!(
    tag!("\"") >>
    x: many0!(alt!(is_not!("\"\\") | tag!("\\\"") | tag!("\\\\") | linebreak_string | tag!("\\r") | tag!("\\b") | tag!("\\f") | tag!("\\t") | take_unicode | tag!("\\DEL") | tag!("\\NUL") | tag!("\\^M") | tag!("\\n"))) >>
    tag!("\"") >>
    opt!(tag!("\n")) >>
    (x)
  ))
);

// parse a line or skip commented line
named_args!(qualifier_substitution<'a>(old: &'a str, old_dot: &'a str, new: &'a str, new_dot: &'a str, special: &'a str)<&'a str, Vec<&'a str>>,
  do_parse!(
    a: many0!(
      alt!(
        do_parse!(a: skip >> (vec![a])) |
        do_parse!(a: call!(interesting_line, old, old_dot, new, new_dot, special) >> (a))
      )
    ) >>
    (join(a))
  )
);

named_args!(pub parse_full<'a>(old: &'a str, old_dot: &'a str, new: &'a str, new_dot: &'a str, special: &'a str)<&'a str, Vec<&'a str>>,
  do_parse!(
    a: recognize!(many0!(
      skip
    )) >>
    b: opt!(complete!(call!(module_name, old, new))) >>
    f: opt!(complete!(call!(parse_import_list, old, new))) >>
    g: call!(qualifier_substitution, old, old_dot, new, new_dot, special) >>
    (join(vec![vec![a], from_vec(b), from_vec(f), g]))
  )
);
