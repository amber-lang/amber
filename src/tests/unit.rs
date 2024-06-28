use crate::tests::comp;

#[test]
fn hello() {
    comp(
        r#"
        echo "hello"
        "#,
        r#"
        echo "hello"
        "#,
    );
}

#[test]
fn basic() {
    comp(
        r#"
        import * from "std"

        // Comment
        let count = 1
        let age = count > 1
            then 18
            else 28

        age += 5
        let name = "John"
        let age = 18
        echo "Hi, I'm {name}. I'm {age} years old."
        "#,
        r#"
        __0_count=1
        __1_age=$(if [ $(echo ${__0_count} '>' 1 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//') != 0 ]; then echo 18; else echo 28; fi)
        __1_age=$(echo ${__1_age} '+' 5 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//')
        __2_name="John"
        __3_age=18
        echo "Hi, I'm ${__2_name}. I'm ${__3_age} years old."
        "#,
    );
}

#[test]
fn basic_import() {
    comp(
        r#"
        import * from "std"
        main {
            echo replace_once("hello world!", "world", "Amber")
        }
        echo "hi"
        "#,
        r#"
        function replace_once__1_v0 {
            local source=$1
            local pattern=$2
            local replacement=$3
            __AMBER_VAL_0=$(echo "${source/${pattern}/${replacement}}");
            __AS=$?;
            __AF_replace_once1_v0="${__AMBER_VAL_0}";
            return 0
        }
        
            replace_once__1_v0 "hello world!" "world" "Amber";
            __AF_replace_once1_v0__4="${__AF_replace_once1_v0}";
            echo "${__AF_replace_once1_v0__4}"
        echo "hi"
        "#,
    );
}
