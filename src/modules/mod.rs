pub mod statement;
pub mod expression;
pub mod block;
pub mod variable;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Text,
    Bool,
    Num,
    Void
}

trait Typed {
    fn get_type(&self) -> Type;
}