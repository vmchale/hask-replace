#![feature(tool_lints)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::double_parens)]

#[macro_use]
extern crate nom;

pub mod cabal;
pub mod hask;
mod test;
mod utils;
