# `docs.ab` and `keywords.ab`

## docs.ab 
Automated script that generates documentation for our standard library (`stdlib`) based on `src/std/` directory

## keywords.ab 
Automatic script which returns alphabetically sorted list of keywords that are used in Amber based on `grammar.ebnf` file

## parse_coverage.ab

Automated script that based on the specific file check the llvm-cov html report `cargo llvm-cov --all-features --workspace --html` what lines are not covered