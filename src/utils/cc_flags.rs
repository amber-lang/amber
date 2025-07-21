use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum CCFlags {
    AllowNestedIfElse,
    AllowGenericReturn,
    AllowAbsurdCast,
    AllowCamelCase,
    UndefinedFlag
}

pub fn get_ccflag_by_name(flag: &str) -> CCFlags {
    match flag {
        "allow_nested_if_else" => CCFlags::AllowNestedIfElse,
        "allow_generic_return" => CCFlags::AllowGenericReturn,
        "allow_camel_case" => CCFlags::AllowCamelCase,
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
        CCFlags::AllowCamelCase => "allow_camel_case",
        CCFlags::UndefinedFlag => "undefined_flag"
    }
}

