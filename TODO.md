# Speed
  - [ ] evaluate the parallelism
  - [ ] look at what tokei does re: reading only parts of a file
# Ergonomics
  - [ ] prompt before replacement
  - [x] vim plugin/wrapper
    - [ ] nice diff output like vim-hare
  - [ ] should optionally move spec files as well
  - [ ] allow user to input module name or file name
  - [ ] git stash instead of git commit? make it automatic, ideally.
# Features
  - [ ] move functions between modules
  - [x] move Idris module
  - [x] move Elm module
  - [x] copy a module
    - [ ] shouldn't do replacements everywhere.
    - [ ] .cabal file needs to be handled better.
  - [ ] move a directory structure
  - [ ] add a module
    - [ ] mustache templates
  - [ ] add a function to the export list if it is present
  ~~- [ ] rename a function across modules~~
# Bugs
  - [x] have to run it from the parent directory
  - [x] Overzealous matches when replacing
  - [ ] shouldn't give package warning for Elm
  - [x] shouldn't generate too many directories
  - [ ] bug with `--copy` and lens.
# Code Maintenance
  - [ ] Replace regular expressions with a real parser
