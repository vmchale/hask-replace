# Speed
  ~~- [ ] evaluate the parallelism~~
# Ergonomics
  - [ ] prompt before replacement
  - [x] vim plugin/wrapper
    - [ ] nice diff output like vim-hare
    - [ ] should optionally move spec files as well
  - [x] should optionally move spec files as well
  ~~- [ ] allow user to input module name or file name~~
  - [ ] git stash instead of git commit? make it automatic, ideally.
# Features
  - [ ] support for backpack (?)
    - [x] support for `.hs-boot` files
  - [x] move Idris module
  - [x] move Elm module
  - [x] copy a module
    - [x] shouldn't do replacements everywhere.
    - [x] .cabal file needs to be handled better.
  - [ ] move a directory structure
  - [ ] new module? Or new test suite perhaps?
  ~~- [ ] add a function to the export list if it is present~~
  ~~- [ ] rename a function across modules~~
# Bugs
  - [x] have to run it from the parent directory
  - [x] Overzealous matches when replacing
  - [ ] shouldn't give package warning for Elm
  - [x] shouldn't generate too many directories
  - [x] bug with `--copy` and lens.
  - [x] fix bug w/ `--copy` and double commas.
# Code Maintenance
  - [ ] Replace regular expressions with a real parser
