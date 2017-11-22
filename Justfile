ci:
    yamllint src/options-en.yml
    yamllint appveyor.yml
    yamllint .travis.yml
    tomlcheck --file Cargo.toml
    cargo check
    cargo test

check:
    git diff master origin/master

# only works if you disable checking that the destination module exists.
bench:
    @rm -rf dhall-1.8.0
    @cabal unpack dhall
    @cargo build --release
    bench "./target/release/hr module dhall-1.8.0 'Dhall.Import' 'Dhall.Import'"
    @rm -rf dhall-1.8.0 lens-4.15.4
    @cabal unpack lens
    bench "./target/release/hr module lens-4.15.4 'Control.Lens.Internal' 'Control.Lens.Internal'"
    @rm -rf lens-4.15.4 haskell-src-exts-1.19.1

packages:
    @rm -rf lens-* idris-lens dhall-* language-lua-*
    @git clone https://github.com/HuwCampbell/idris-lens.git
    cd idris-lens && cargo run -- idris . Control.Lens.Maths Control.Lens.Math && idris --build lens.ipkg
    @rm -rf idris-lens
    @cabal unpack language-lua
    cargo run -- module language-lua-0.10.0 Language.Lua.Annotated.Parser Language.Lua.Annotate.ParserAgain && cd language-lua-0.10.0 && cabal new-build
    @rm -rf language-lua-0.10.0
    @cabal unpack dhall
    cargo run -- module dhall-1.8.0 "Dhall.Import" "Dhall.Dependencies" && cd dhall-1.8.0 && cabal new-build
    @rm -rf dhall-1.8.0
    @cabal unpack lens
    cd lens-4.15.4 && cargo run -- module . "Control.Lens.Internal" "Control.Lens.Mine" --copy && cabal new-build
    @rm -rf lens-4.15.4
    @git clone https://github.com/debois/elm-mdl
    cd elm-mdl && cargo run -- elm . Material.Options.Internal Material.Options.Mod && elm-make --yes
    @rm -rf elm-mdl/

test:
    @rm -rf nothing
    @pi new elm nothing
    cargo run -- elm nothing "Update" "Update.Nested.Mod"
    cd nothing && elm-make src/main.elm --yes
    @rm -rf nothing/ test-nothing
    @pi new haskell test-nothing
    cargo run -- module test-nothing "Lib" "NewLib.Nested"
    cd test-nothing && cabal new-test
    @rm -rf test-nothing nothing
    @pi new idris nothing
    cargo run -- idris nothing "Nothing.Lib" "NewLib.Nested"
    cd nothing && idris --build nothing.ipkg
    @rm -rf nothing

patch:
    cargo release -l patch --no-dev-version

minor:
    cargo release -l minor --no-dev-version
