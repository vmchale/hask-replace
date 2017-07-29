# hask-replace

`hask-replace` is a command-line tool that simplifies the process of renaming
Haskell modules.

## The Pitch

Here's an example of how you would use `hr`:

```bash
cabal unpack dhall
cd dhall-1.5.0
hr module . "Dhall.Import" "Dhall.Dependencies"
cabal new-build
```

As you can see, it's a lot less painful than whatever witchcraft using bash and
`sed` would accomplish the same thing.

## Installation

### Binary releases

The easiest way for most users is simply to download the prebuilt binaries.
You can find binaries for various platforms on the
[release](https://github.com/vmchale/hask-replace/releases) page.

### Cargo

First, install [cargo](https://rustup.rs/). Then:

```bash
 $ cargo install hask-replace
```

You will need to use the nightly release for this to work; if in doubt run

```bash
rustup run nightly cargo install hask-replace
```

## Use
