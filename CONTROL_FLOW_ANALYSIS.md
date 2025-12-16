Control flow analysis is a compile-time optimization that evaluates boolean conditions to eliminate dead code and narrow variable types. This enables the compiler to skip typechecking and code generation for branches that will never execute. This PR introduces two new mechanisms:
- Constant folding (`analyze_control_flow`)
- Type narrowing (`extract_facts`)

This PR updates control flow structures (`ifelse`, `ifchain`, `ternary`) to use the new mechanisms.

### Constant Folding

The compiler evaluates boolean expressions at compile time when possible:

```amber
if false {
    // This code is never typechecked or compiled
    undefined_function()
}
```

Each expression type implements `analyze_control_flow` that returns `Option<bool>`:
- `Some(bool)` if the expression can be evaluated at compile time
- `None` if the expression depends on runtime values

| Expression | Behavior |
|------------|----------|
| `true` / `false` | Returns the literal value |
| `A and B` | Returns `false` if either is `false`, `true` only if both are `true` |
| `A or B` | Returns `true` if either is `true`, `false` only if both are `false` |
| `not A` | Inverts the result |
| `x is T` | Returns `true` if types match, `false` if types cannot intersect |
| `(expr)` | Delegates to inner expression |

### Type Narrowing

When using the `is` operator, the compiler narrows variable types within conditional blocks:

```amber
fun example(x: Text | Int) {
    if x is Int {
        // x is treated as Int here
        echo x + 1
    }
}
```

Each expression type implements `extract_facts` that returns a tuple of `true_facts` and `false_facts`:
- `true_facts` are the facts that hold true when the expression is true
- `false_facts` are the facts that hold true when the expression is false

| Expression | True-Facts | False-Facts |
|------------|------------|-------------|
| `x is T` | `{x: T}` | `{}` |
| `A and B` | Merge both | Intersect both |
| `A or B` | Intersect both | Merge both |
| `not A` | A's false-facts | A's true-facts |

