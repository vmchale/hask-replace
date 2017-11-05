# Speed
  - [ ] fix parser as much as possible
# Ergonomics
  - [ ] prompt before replacement
  - [ ] optionally proceed if no config file is found.
  - [ ] if we replace a `parser.cabal` file, and it's in a directory called
    `parser`, we should rename the directory too.
  - [x] vim plugin/wrapper
    - [ ] nice diff output like vim-hare
    - [ ] should optionally move spec files as well
# Features
  - [ ] support for backpack (?)
  - [ ] run regex on .hlint.yaml file as well.
    - [x] support for `.hs-boot` files
    - [x] support for alex/happy
    - [ ] support for `.hsig` files
    - [ ] parse cabal file to find source directories.
  - [ ] move a directory structure
  - [ ] new module? Or new test suite perhaps?
# Bugs
  - [ ] fails on `idris-lens`
  - [ ] module copying/cabal file
# Code Maintenance
