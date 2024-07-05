<img src="assets/amber.png" alt="amber logo" width="250" align="right" />

# Contributing to Amber
This is a simple but exhaustive guide to get you started on contributing to amber.

## Contributing guidelines
Before you dig into Amber, you should know a few things before you contribute.

Any code change is submitted [through a PR](https://github.com/Ph0enixKM/Amber/pulls), which is then approved by at least 2 maintainers.

The way we talk on github is not the same as we would talk in person. When on github, always get straight to the point and be critical.

Personal grudges are forbidden around here, as well as anything offtopic or offensive.

### Opening a PR

Before a PR is opened, it usually has an issue about it first, where we discuss how exactly a feature must be implemented, to avoid making a mistake.

It is recommended that you see how features were already implemented. A good example is [#130](https://github.com/Ph0enixKM/Amber/issues/130)

To create a PR, you should fork the repo, create a branch, do your work in there, and open a PR. It will then be reviewed and pushed into master.

### Getting help
Along the way, you may need help with your code. The best way to ask is in [our Discord server](https://discord.com/invite/cjHjxbsDvZ), but you may also ask other contributors personally or post in [Discussions](https://github.com/Ph0enixKM/Amber/discussions).

## Overview
Amber consists of the following layers:

1. [CLI Interface](#1-cli-interface)
2. [Compiler](#2-compiler)  
   1. [Parser & tokenizer](#21-parser--tokenizer)
   2. [Translator](#22-translator)
3. [Runtime libraries](#3-runtime-libraries)
   1. [`stdlib`](#31-stdlib)
4. [Tests](#4-tests)

### 1. CLI Interface
All CLI interface is in [`main.rs`](src/main.rs). [`clap`](https://crates.io/crates/clap) handles argument parsing.

### 2. Compiler
Compiler consists of:
- [`compiler.rs`](src/compiler.rs) - Main entry point for the compiler
- [`rules.rs`](src/rules.rs) - Syntax rules that are used by Heraclitus framework to correctly output tokens
- [`utils`](src/utils.rs) - Contains parsing environments, caches, contexts and Amber's implementations of metadata
- [`modules`](src/modules) - Syntax modules that parse Amber syntax and also handle the translation process
- [`translate`](src/translate) - Contains a definition of Translate Module trait that is used to translate modules the previously mentioned `modules`

`AmberCompiler` struct by itself is just a bootstrapper for all the syntax modules.

#### 2.1. Parser & tokenizer
Thanks to [`heraclitus`](https://github.com/Ph0enixKM/Heraclitus), we can use simple abstractions to go through tokens.

Please open any syntax module code file, and find a line that says: `impl SyntaxModule<ParserMetadata> for MODULE_NAME_HERE`

It will have a `parse()` function, where all the magic happens. You can either dig into the code yourself or look at the example below to understand how it works.

<details>
<summary>Example parser</summary>

**Important: this is pseudo code. Its purpose is to demonstrate how it should look like.**

```rs
// This code parses the following: `1 + 2`
fn parse(meta: &mut ParserMetadata) -> SyntaxResult {
    let digit_1 = meta.get_current_token();     // gets the text (as an Option)
    token(meta, "+")?;                          // matches that there is a "+" and skips it
    let digit_2 = meta.get_current_token();

    self.digit_1 = digit_1.unwrap();
    self.digit_2 = digit_2.unwrap();

    Ok(())
}
```
</details>

#### 2.2. Translator
Same as parser open a syntax module, and find a line that says `impl TranslateModule for MODULE_NAME_HERE` and that should contain a `translate` function.

Same as before, you can either dig into the code you opened or look at the example below.

<details>
<summary>Example parser</summary>

**Important: this is pseudo code. Its purpose is to demonstrate how it should look like.**

```rs
// This will translate `1 + 2` into `(( 1 + 2 ))`
fn translate() -> String {

    // self.digit_1 and self.digit_2 is set earlier by the parser
    format!("(( {} + {} ))", self.digit_1, self.digit_2)
}
```
</details>

Basically, the `translate()` method should return a `String` for the compiler to construct a compiled file from all of them. If it translates to nothing, you should output an empty string, like `String::new()`

### 3. Runtime libraries
#### 3.1. `stdlib`

`stdlib` is written in Amber. See [`main.ab`](src/std/main) for the code. All `stdlib` functions must be covered by a [test](#4-tests)

### 4. Tests
Amber uses `cargo test` for tests. `stdlib` and `validity` tests usually work by executing amber code and checking its output.

We have [`validity tests`](src/tests/validity.rs) to check the compiler, [`stdlib tests`](src/tests/stdlib.rs) and [`CLI tests`](src/tests/cli.rs).  

The majority of `stdlib` tests are Written in pure Amber in the folder [`tests/stdlib`](src/tests/stdlib). For every test there is a `*.output.txt` file that contains the expected output.
Tests will be executed without recompilation. Amber will load the scripts and verify the output in the designated file to determine if the test passes.

A part of those tests like for `download` require Rust to load a web server, so there is another folder [`tests/stdlib/no_output`](src/tests/stdlib/no_output) that include just the Amber script, the output is inside the [`stdlib tests`](src/tests/stdlib.rs) file.

<details>
<summary>Let's write a simple test</summary>

```rs
#[test]
fn prints_hi() {
    let code = "
        echo \"hi!\"
    ";
    test_amber!(code, "hi!");
}
```
</details>

#### Running tests
To run ALL tests, run `cargo test`.

If you want to run only tests from a specific file, let's say from [`stdlib.rs`](src/tests/stdlib.rs), you add the file name to the command: `cargo test stdlib`

And if there is a specific function, like `test_function()` in `stdlib.rs`, you should add the full path to it: `cargo test stdlib::test_function`

