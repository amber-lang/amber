### Summary
Implement the `nameof` functionality for functions in Amber, as described in issue #673 and the proposed syntax in the comments.

### Details
- When used without arguments, `nameof` should return the name of the function. Example:
  ```amber
  fun foo() {
    echo "Hello World"
  }

  echo nameof(foo)  // Returns the compiled name of `foo`
  ```

- For functions with arguments, support specifying argument types explicitly. Example:
  ```amber
  fun foo(arg) {
    echo arg
  }
  echo nameof foo(Text)  // Returns the name of `foo` that accepts a Text parameter

  fun bar(arg1, arg2) {
    echo arg1 + arg2
  }
  echo nameof bar(Int, Int)  // Returns the name of `bar` for Int, Int arguments
  ```

### Motivation
This feature is essential for enabling traps and other advanced functionality that relies on retrieving function names dynamically.

Please refer to the linked issue and comments for additional context.