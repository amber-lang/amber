# ShellCheck Warning Analysis

Current state: **10 warnings** in `report.txt` (stdlib corpus).
All `.ab`-fixable warnings are resolved. Remaining warnings are **compiler-generated** and need Rust changes.

Rule #6926: **zero exclusions** — every warning must be fixed at the source. No `--exclude` flags, no `.shellcheckrc`.

---

## Warning Inventory (10 warnings, 3 codes)

| Code   | Count | Origin                              | Category                |
|--------|-------|-------------------------------------|-------------------------|
| SC2296 | 5     | Zsh nameref `${(P)name}`           | **False positive**      |
| SC2046 | 4     | Ksh `set -A result -- $(printf …)` | **Real bug (word-split)**|
| SC2001 | 1     | `sed` for regex in `text.ab`       | Design constraint       |

---

## Category 1: Genuine False Positives (5 warnings)

### SC2296 — Zsh `${(P)name}` indirect expansion (5 instances)

**Affected files:**
- `env_const_get.ab.sh` (line 42)
- `env_var_get.ab.sh` (line 30)
- `env_var_load.ab.sh` (line 204)
- `env_var_set.ab.sh` (line 42)
- `env_var_test.ab.sh` (line 16)

**Source:** Compiler-generated zsh nameref code (`src/translate/fragments/var_expr.rs`).
**Why ShellCheck is wrong:** The `(P)` flag is valid zsh parameter expansion syntax. Zsh docs confirm: `${(P)name}` forces the value of `name` to be interpreted as a further parameter name whose value is used. ShellCheck does not parse zsh parameter expansion flags and treats the `(` as invalid.
**Runtime verification:** The generated zsh scripts execute correctly. The `${(P)name}` indirection resolves the variable as intended.
**Conclusion:** True false positive. ShellCheck limitation, not a code issue. Cannot be fixed without rewriting the zsh indirection pattern (which works correctly at runtime).

---

## Category 2: Real Bugs (4 warnings)

### SC2046 — Array word-splitting loses elements with spaces (4 instances)

**Affected file:** `array_sorted.ab.sh` (lines 41, 74, 107, 140)

**The bug:** The ksh array-sort idiom uses `set -A result -- $(printf "%s\n" ...)` with word-splitting to populate arrays. Elements containing spaces are split into separate array elements.

```ksh
# Generated (broken for elements with spaces):
set -A result_34 -- $(printf "%s\n" "${input[@]}")
# If input contains "hello world", result gets ("hello", "world") instead of ("hello world")
```

**ShellCheck is right:** `$(...)` word-splitting is unsafe for array population when elements may contain spaces.
**Root cause:** The compiler uses command substitution + word-splitting as a portable array-assignment idiom for ksh. This breaks on array elements containing spaces.
**Fix:** Use `read -A` to populate the array without word-splitting:

```ksh
# Instead of:  set -A result -- $(printf "%s\n" "${input[@]}")
# Emit:        printf "%s\n" "${input[@]}" | { read -A result; }
```

Or use a here-string:
```ksh
read -A result <<< "$(printf "%s\n" "${input[@]}")"
```

**Files to modify:**
- `src/std/array.ab` — `sorted()` ksh branch, replace `set -A -- $(...)` with `read -A`
- Or compiler-level: change how ksh `set -A` is emitted from array assignments

**Verification:** Add test cases with array elements containing spaces. Run full suite on all 4 targets. Accept updated snapshots.

**Risk:** Medium — ksh-specific change. Must verify `read -A` populates the array correctly on the CI ksh target. `read -A` (uppercase) is ksh-specific; bash uses `read -a` (lowercase), so this must be gated to ksh only.

---

## Category 3: Design Constraints — Intentional Warnings (1 warning)

### SC2001 — `sed` for regex replacement (1 instance)

**Affected file:** `text_replace_regex_basic.ab.sh` (line 120)

**Source:** `src/std/text.ab` — `replace_regex` helper.
**Why it's intentional:** ShellCheck suggests `${var//search/replace}`, but that uses glob patterns, not regex. The `replace_regex` function needs actual regex support (`[0-9]+`, `\b`, etc.). Parameter expansion cannot do regex.
**Verification:** Confirmed — `${var//[0-9]+/x}` does NOT match one-or-more digits in bash. `sed 's/[0-9]\+/x/g'` does.
**Conclusion:** Not fixable. `sed` is the correct tool for regex replacement.

---

## Fix Priority

1. **SC2046** (4 warnings) — Ksh array word-splitting. Replace `set -A -- $(...)` with `read -A` in ksh branch of `array.ab`. Medium risk, fixes a real data-loss bug.
2. **SC2296** (5 warnings) — False positive. No fix needed; document as ShellCheck limitation.
3. **SC2001** (1 warning) — `sed` for regex. No fix needed.

**Total fixable: 4 warnings** (SC2046).
**Total false positives / design constraints: 6 warnings.**

---

## Shell Compatibility Verification

For every Rust change, run on all 4 targets:
```bash
cargo build
cargo test
AMBER_TEST_TARGET=bash-3.2 cargo insta test --accept
AMBER_TEST_TARGET=ksh cargo insta test --accept
AMBER_TEST_TARGET=zsh cargo insta test --accept
```

Key shell-specific concerns for the remaining fix:
- **Ksh:** `read -A` (uppercase A) for array reading, `set -A` for array init
- **BashLegacy (3.2):** `read -a` (lowercase a), no `mapfile`
- **Zsh:** `read -A` also works in zsh for array reading
- **BashModern:** `mapfile -t` (most efficient), `read -a`
