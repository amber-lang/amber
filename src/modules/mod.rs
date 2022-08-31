pub mod statement;
pub mod expression;
pub mod block;
pub mod variable;
pub mod command;
pub mod conditions;
pub mod shorthand;
pub mod loops;

#[macro_export]
macro_rules! handle_types {
    ($enum_name:ident, [$($item:tt),*]) => {
        fn get_modules(&self) -> Vec<$enum_name> {
            vec![
                $(
                    $enum_name::$item($item::new())
                ),*
            ]
        }

        fn parse_match(&mut self, meta: &mut ParserMetadata, module: $enum_name) -> SyntaxResult {
            match module {
                $(
                    $enum_name::$item(module) => self.get(meta, module, $enum_name::$item)
                ),*
            }
        }

        fn translate_match(&self, meta: &mut TranslateMetadata, module: &$enum_name) -> String {
            match module {
                $(
                    $enum_name::$item(module) => module.translate(meta)
                ),*
            }
        }
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Text,
    Bool,
    Num,
    Null
}

pub trait Typed {
    fn get_type(&self) -> Type;
}