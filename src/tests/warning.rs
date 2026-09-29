/// Tests for Amber scripts that check for warning messages.
use super::script_test;
use super::TestOutcomeTarget;
use test_generator::test_resources;

/// Autoload the Amber test files in warning, match the output in the comment
#[test_resources("src/tests/warning/*.ab")]
fn test_warning(input: &str) {
    script_test(input, TestOutcomeTarget::Success);
}

#[test]
fn target_dependent_dead_code_warnings() {
    use crate::compiler::{AmberCompiler, CompilerOptions};
    use crate::utils::ShellType;

    for target in [ShellType::BashModern, ShellType::BashLegacy, ShellType::Zsh, ShellType::Ksh] {
        for (condition, expected_warnings) in [
            (r#"shellname() == "bash""#, 0),
            (r#"shellname() != "bash""#, 0),
            (r#"not (shellname() == "bash")"#, 0),
            (r#"true and (shellname() == "bash")"#, 0),
            (r#"false or (shellname() == "bash")"#, 0),
            (r#"(shellname() == "bash") and false"#, 1),
            (r#"false and (shellname() == "bash")"#, 1),
            (r#"(shellname() == "bash") or true"#, 1),
            (r#"true or (shellname() == "bash")"#, 1),
        ] {
            let code = format!(
                "if {condition} {{ echo(\"yes\") }} else {{ echo(\"no\") }}"
            );
            let options = CompilerOptions::default().with_target(Some(target));
            let (warnings, _) = AmberCompiler::new(code, None, options).compile().unwrap();
            assert_eq!(warnings.len(), expected_warnings, "{condition} on {target:?}");
        }

        let code = r#"if {
            shellname() == "bash": echo("bash")
            shellname() == "zsh": echo("zsh")
            shellname() == "ksh": echo("ksh")
            else { echo("other") }
        }"#;
        let options = CompilerOptions::default().with_target(Some(target));
        let (warnings, _) = AmberCompiler::new(code.into(), None, options).compile().unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
    }
}
