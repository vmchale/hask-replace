check:
    git diff master origin/master

packages: 
    rm -rf lens-* idris-lens dhall-*
    cabal unpack dhall
    cd dhall-1.7.0 && cargo run -- module . "Dhall.Import" "Dhall.Dependencies" && cabal new-build
    rm -rf dhall-1.7.0
    cabal unpack lens
    cd lens-4.15.4 && cargo run -- module . "Control.Lens.Internal" "Control.Lens.Mine" --copy && cabal new-build
    rm -rf lens-4.15.4

#git clone https://github.com/HuwCampbell/idris-lens.git
#cd idris-lens && hr idris . Control.Lens.Maths Control.Lens.Math && idris --build lens.ipkg
#rm -rf idris-lens
#git clone https://github.com/debois/elm-mdl
#cd elm-mdl && hr elm . Material.Options.Internal Material.Options.Mod && elm-make --yes
#rm -rf elm-mdl/

test:
    rm -rf nothing
    pi new elm nothing
    cargo run -- elm nothing "Update" "Update.Nested.Mod"
    cd nothing && elm-make src/main.elm --yes
    rm -rf nothing/ test-nothing
    pi new haskell test-nothing
    cargo run -- module test-nothing "Lib" "NewLib.Nested"
    cd test-nothing && cabal new-test
    rm -rf test-nothing nothing
    pi new idris nothing
    cargo run -- idris nothing "Nothing.Lib" "NewLib.Nested"
    cd nothing && idris --build nothing.ipkg
    rm -rf nothing

patch:
    cargo release -l patch --no-dev-version

minor:
    cargo release -l minor --no-dev-version
