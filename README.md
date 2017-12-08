# hask-replace

[![Windows build status](https://ci.appveyor.com/api/projects/status/github/vmchale/hask-replace?svg=true)](https://ci.appveyor.com/project/vmchale/hask-replace)
[![Build Status](https://travis-ci.org/vmchale/hask-replace.svg?branch=master)](https://travis-ci.org/vmchale/hask-replace)

`hask-replace` is a command-line tool for renaming
Haskell, Elm, PureScript, and Idris modules.

## The Pitch

Here's an example of how you would use `hr`:

```bash
cabal unpack dhall
cd dhall-1.5.1/
hr module . "Dhall.Import" "Dhall.Dependencies"
cabal new-build
```

As you can see, it's a lot less painful than whatever witchcraft you'd have to
resort to to accomplish the same thing in bash.

## The Anti-Pitch

`hr` doesn't attempt to be a full parser for `cabal`, `idris`, `elm`, etc. While
`hr` promises to always transform *valid* projects into valid projects, it won't
necessarily point out errors in your project.

## Installation

### Script

The easiest way to install for most users is probably via a shell script, viz.

```
curl -LSfs https://japaric.github.io/trust/install.sh | sh -s -- --git vmchale/hask-replace
```

### Binary releases

If the script doesn't work, you can also download prebuilt binaries.
You can find binaries for various platforms on the
[release](https://github.com/vmchale/hask-replace/releases) page.

### Cargo

First, install [cargo](https://rustup.rs/). Then:

```bash
 $ cargo install --git https://github.com/vmchale/hask-replace
```

You will need to use the nightly release for this to work; if in doubt run

```bash
 $ rustup run nightly cargo install --git https://github.com/vmchale/hask-replace
```

## Performance

| Package | Task | Time |
| ------- | ---- | ---- |
| dhall | Rename Module | 7.185 ms |
| lens | Rename Module | 9.671 ms |

## Use

Example use:

```bash
git clone https://github.com/HuwCampbell/idris-lens.git
cd idris-lens
hr idris . Control.Lens.Maths Control.Lens.Math
idris --build lens.ipkg
```

### Vim Plugin

There is a vim plugin for hask-replace
[here](https://github.com/vmchale/hask-replace-vim). It supports copying and
moving Haskell, Elm, and Idris modules.
