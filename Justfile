ci:
    yamllint src/options-en.yml
    yamllint appveyor.yml
    yamllint .travis.yml
    tomlcheck --file Cargo.toml
    cargo check
    cargo test

check:
    git diff master origin/master

bench:
    @rm -rf dhall-1.9.1
    @cabal unpack dhall
    @cargo build --release
    bench "./target/release/hr module dhall-1.9.1 'Dhall.Import' 'Dhall.Import' --benchmark-mode"
    @rm -rf dhall-1.9.1 lens-4.16
    @cabal unpack lens
    bench "./target/release/hr module lens-4.16 'Control.Lens.Internal' 'Control.Lens.Internal' --benchmark-mode"
    @rm -rf lens-4.16 haskell-src-exts-1.19.1

packages:
    @rm -rf cabal
    @git clone https://github.com/haskell/cabal && cd cabal && cargo run -- m . Distribution.Backpack Distribution.FannyPack && cabal new-build all
    @rm -rf purescript-matryoshka
    @git clone https://github.com/slamdata/purescript-matryoshka.git
    cd purescript-matryoshka && cargo run -- p . Matryoshka.DistributiveLaw Matryoshka.DL && npm install && bower install && npm run -s build && npm run -s test
    @rm -rf cabal
    @git clone https://github.com/diku-dk/futhark
    cd futhark && cargo run -- m . Language.Futhark.Parser.Parser Language.Futhark.Parser.Mod --hpack && cargo run -- m . Language.Futhark.TH Language.Futhark.Sin --hpack && stack build
    @rm -rf futhark
    @git clone https://github.com/HuwCampbell/idris-lens.git
    cd idris-lens && cargo run -- idris . Control.Lens.Maths Control.Lens.Math && idris --build lens.ipkg
    @rm -rf idris-lens
    @cabal unpack language-lua
    cargo run -- module language-lua-0.10.0 Language.Lua.Annotated.Parser Language.Lua.Annotate.ParserAgain && cd language-lua-0.10.0 && cabal new-build -w ghc-8.2.2
    @rm -rf language-lua-0.10.0
    @cabal unpack dhall
    cargo run -- module dhall-1.8.2 "Dhall.Import" "Dhall.Dependencies" && cd dhall-1.8.2 && cabal new-build -w ghc-8.2.2
    @rm -rf dhall-1.8.2
    @cabal unpack lens
    cd lens-4.16 && cargo run -- module . "Control.Lens.Internal" "Control.Lens.Mine" --copy && cabal new-build -w ghc-8.2.2
    @rm -rf lens-4.16
    @rm -rf lens-* idris-lens dhall-* language-lua-* purescript-matryoshka futhark cabal

test:
    @rm -rf nothing
    @rm -rf nothing/ test-nothing
    @pi new haskell test-nothing
    cargo run -- module test-nothing "Lib" "NewLib.Nested"
    cd test-nothing && cabal new-test -w ghc-8.2.2
    @rm -rf test-nothing nothing
    @pi new idris nothing
    cd ~/programming/idris/depends/Idris-Bifunctors/ && idris --install bifunctors.ipkg
    cargo run -- idris nothing "Nothing.Lib" "NewLib.Nested"
    cd nothing && idris --build nothing.ipkg
    @rm -rf nothing

patch:
    cargo release -l patch --no-dev-version

minor:
    cargo release -l minor --no-dev-version
