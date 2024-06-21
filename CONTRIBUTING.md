<img src="assets/amber.png" alt="amber logo" width="200" align="right" />

# Contributing to Amber
This is a simple but exhaustive guide to get you started on contributing to amber.

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
Compiler consists of these files: [`compiler.rs`](src/compiler.rs), and everything in [`modules`](src/modules).

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
fn parse(meta: ParserMetadata) {
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
Amber uses `cargo test` for tests. They are written in rust, and they use a `test_amber` macro.

We have [`validity tests`](src/tests/validity.rs) to check the compiler, [`stdlib tests`](src/tests/stdlib.rs) and [`CLI tests`](src/tests/cli.rs).

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