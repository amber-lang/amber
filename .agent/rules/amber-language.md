---
trigger: always_on
---

# Important information before you start:
- To understand how the language works, read grammar.ebnf file that is in the project root.
- There are different types of tests. You can see them all in src/tests/ directory. If test is written in .ab file, then it matches it's execution output with `// Output` comment section.

For instance:
```
// Outputs
// 5

echo 5
```
This example will succeed, where this one will fail:
```
// Outputs
// 10

echo 5 // Outputs expects it to be 10 and not 5
```
- You can test the functionality of the compiler by running `cargo test`. You can narrow down the testing to only check vailidty tests for instance with `cargo test validity`.
- Please do cleanup in the end, when finished working. If you do it in the middle, I have to confirm each `rm` call, because it's potentially destructive. This helps me a lot!
- Try to just use `pub` and avoid using `pub(crate)`

## The requirement is that you should always create tests if any of these conditions are met:
- A new syntax behaviour is introduced / changed => add new `validity` tests
- A new and simple mechanic (lower IR is less than 300 lines of code) that is added that changes the way we translate code => add new `translating` tests
- We add new mechanics that alter how the Bash is compiled => add new `erroring` tests
- We add optimization layer in src/optimizer => add new `optimizing` tests
- We add new functions to stdlib / new edge cases are fixed for stdlib => add new `stdlib` tests
- If there are new error messages introduced / removed or the mechanics around it has changed => add new `erroring` tests
- If there are new warning messages introduced / removed or the mechanics around it has changed => add new `warning` tests