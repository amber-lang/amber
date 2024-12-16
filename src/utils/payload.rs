use crate::modules::builtin::cli::param::ParamImpl;
use crate::modules::builtin::cli::parser::ParserImpl;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum Payload {
    Parser(Rc<RefCell<ParserImpl>>),
    Param(Rc<RefCell<ParamImpl>>),
}

impl Payload {
    pub fn set_var_name(&mut self, name: &str, id: Option<usize>) {
        match self {
            Payload::Param(param) => param.borrow_mut().set_var_name(name, id),
            _ => (),
        }
    }
}
