pub mod flag_registry;
pub mod cli;

#[cfg(test)]
mod tests {
    use super::cli::CLI;

    #[test]
    fn hello_world() {
        let cli = CLI::new();
        assert_eq!(cli.test_eval("$echo Hello World$").trim(), "Hello World");
    }

    #[test]
    fn add() {
        let cli = CLI::new();
        assert_eq!(cli.test_eval("$echo {15 + 45}$").trim(), "60");
    }

    #[test]
    fn mul() {
        let cli = CLI::new();
        assert_eq!(cli.test_eval("$echo {3 * 4}$").trim(), "12");
    }

    #[test]
    fn div() {
        let cli = CLI::new();
        assert_eq!(cli.test_eval("$echo {21 / 3}$").trim(), "7");
    }

    #[test]
    fn sub() {
        let cli = CLI::new();
        assert_eq!(cli.test_eval("$echo {21 - 7}$").trim(), "14");
    }

    #[test]
    fn text() {
        let cli = CLI::new();
        assert_eq!(cli.test_eval("$echo {'Hello World'}$").trim(), "Hello World");
    }

    #[test]
    fn bool() {
        let cli = CLI::new();
        assert_eq!(cli.test_eval("$echo {true}$").trim(), "1");
        assert_eq!(cli.test_eval("$echo {false}$").trim(), "0");
    }

    #[test]
    fn number() {
        let cli = CLI::new();
        assert_eq!(cli.test_eval("$echo {42}$").trim(), "42");
    }

    #[test]
    fn variable() {
        let cli = CLI::new();
        let code = "
            let x = 42
            $echo {x}$
            x = 21
            $echo {x}$
        ";
        assert_eq!(cli.test_eval(code).trim(), "42\n21");
    }

    #[test]
    fn nested_string_interp() {
        let cli = CLI::new();
        let code = "
            let x = 'welcome {'to'} wonderful {'world'}'
            $echo {x}$
        ";
        assert_eq!(cli.test_eval(code).trim(), "welcome to wonderful world");
    }

    #[test]
    fn complex_arithmetic() {
        let cli = CLI::new();
        let code = "
            let x = 21
            let y = 2
            let z = 3
            $echo {x + y * z}$
        ";
        assert_eq!(cli.test_eval(code).trim(), "27");
    }

    #[test]
    fn very_complex_arithmetic() {
        let cli = CLI::new();
        let code = "
            let x = 21
            let y = 2
            let z = 6
            let a = 4
            $echo {x + y * z / a}$
        ";
        assert_eq!(cli.test_eval(code).trim(), "24");
    }

    #[test]
    fn paranthesis() {
        let cli = CLI::new();
        let code = "
            let x = 21
            let y = 2
            let z = 6
            let a = 2
            $echo {(x + y) * z / a}$
        ";
        assert_eq!(cli.test_eval(code).trim(), "69");
    }

    #[test]
    fn command_interpolation() {
        let cli = CLI::new();
        let code = "
            $echo {$echo {$echo Hello World$}$}$
        ";
        assert_eq!(cli.test_eval(code).trim(), "Hello World");
    }

    #[test]
    fn comment() {
        let cli = CLI::new();
        let code = "
            # this is a comment
            let a = 42 # this is a comment as well
        ";
        assert_eq!(cli.test_eval(code).trim(), "");
    }

    #[test]
    fn compare_eq_texts() {
        let cli = CLI::new();
        let code = "
            let x = 'Hello World'
            let y = 'Hello World'
            $echo {x == y}$
        ";
        assert_eq!(cli.test_eval(code).trim(), "1");
    }

    #[test]
    fn compare_eq_numbers() {
        let cli = CLI::new();
        let code = "
            let x = 42
            let y = 42
            $echo {x == y}$
        ";
        assert_eq!(cli.test_eval(code).trim(), "1");
    }

    #[test]
    fn compare_neq_numbers() {
        let cli = CLI::new();
        let code = "
            let x = 42
            let y = 24
            $echo {x != y}$
        ";
        assert_eq!(cli.test_eval(code).trim(), "1");
    }
}