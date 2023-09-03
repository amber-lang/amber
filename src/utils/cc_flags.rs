#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum CCFlags {
    AllowNestedIfElse,
    AllowGenericReturn,
    AllowAbsurdCast,
    UndefinedFlag
}

pub fn get_ccflag_by_name(flag: &str) -> CCFlags {
    match flag {
        "allow_nested_if_else" => CCFlags::AllowNestedIfElse,
        "allow_generic_return" => CCFlags::AllowGenericReturn,
        "allow_absurd_cast" => CCFlags::AllowAbsurdCast,
        _ => CCFlags::UndefinedFlag
    }
}

#[allow(dead_code)]
pub fn get_ccflag_name(flag: CCFlags) -> &'static str {
    match flag {
        CCFlags::AllowNestedIfElse => "allow_nested_if_else",
        CCFlags::AllowGenericReturn => "allow_generic_return",
        CCFlags::AllowAbsurdCast => "allow_absurd_cast",
        CCFlags::UndefinedFlag => "undefined_flag"
    }
}

