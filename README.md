# hask-replace

[![Windows build status](https://ci.appveyor.com/api/projects/status/github/vmchale/hask-replace?svg=true)](https://ci.appveyor.com/project/vmchale/hask-replace)
[![Build Status](https://travis-ci.org/vmchale/hask-replace.svg?branch=master)](https://travis-ci.org/vmchale/hask-replace)

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

As you can see, it's a lot less painful than whatever witchcraft you'd have to
resort to to accomplish the same thing in bash.

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
 $ cargo install hask-replace
```

You will need to use the nightly release for this to work; if in doubt run

```bash
rustup run nightly cargo install hask-replace
```

## Use
