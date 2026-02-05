#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum CCFlags {
    AllowNestedIfElse,
    AllowAbsurdCast,
    AllowCamelCase,
    AllowDeadCode,
    AllowPublicMutable,
    UndefinedFlag,
}

pub fn get_ccflag_by_name(flag: &str) -> CCFlags {
    match flag {
        "allow_nested_if_else" => CCFlags::AllowNestedIfElse,
        "allow_camel_case" => CCFlags::AllowCamelCase,
        "allow_absurd_cast" => CCFlags::AllowAbsurdCast,
        "allow_dead_code" => CCFlags::AllowDeadCode,
        "allow_public_mutable" => CCFlags::AllowPublicMutable,
        _ => CCFlags::UndefinedFlag,
    }
}

#[allow(dead_code)]
pub fn get_ccflag_name(flag: CCFlags) -> &'static str {
    match flag {
        CCFlags::AllowNestedIfElse => "allow_nested_if_else",
        CCFlags::AllowAbsurdCast => "allow_absurd_cast",
        CCFlags::AllowCamelCase => "allow_camel_case",
        CCFlags::AllowDeadCode => "allow_dead_code",
        CCFlags::AllowPublicMutable => "allow_public_mutable",
        CCFlags::UndefinedFlag => "undefined_flag",
    }
}
