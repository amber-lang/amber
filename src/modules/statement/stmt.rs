use super::comment::Comment;
use super::comment_doc::CommentDoc;
use crate::docs::module::DocumentationModule;
use crate::modules::builtin::{
    cd::Cd, clear::Clear, cp::Cp, disown::Disown, echo::Echo, exit::Exit, mv::Mv, rm::Rm,
    sleep::Sleep, touch::Touch, wait::Await,
};
use crate::modules::command::cmd::Command;
use crate::modules::command::modifier::CommandModifier;
use crate::modules::condition::{ifchain::IfChain, ifcond::IfCondition};
use crate::modules::expression::expr::Expr;
use crate::modules::function::{declaration::FunctionDeclaration, fail::Fail, ret::Return};
use crate::modules::imports::import::Import;
use crate::modules::loops::{
    break_stmt::Break, continue_stmt::Continue, infinite_loop::InfiniteLoop, iter_loop::IterLoop,
    while_loop::WhileLoop,
};
use crate::modules::main::Main;
use crate::modules::prelude::*;
use crate::modules::shorthand::{
    add::ShorthandAdd, div::ShorthandDiv, modulo::ShorthandModulo, mul::ShorthandMul,
    sub::ShorthandSub,
};
use crate::modules::test::Test;
use crate::modules::variable::{
    init::VariableInit, init_destruct::VariableInitDestruct, set::VariableSet,
    set_destruct::VariableSetDestruct,
};
use crate::parse_statement;
use crate::translate::module::TranslateModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use amber_meta::StatementDispatch;
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone, StatementDispatch)]
pub enum StmtType {
    #[dispatch(translate_discard)]
    Expr(Expr),
    VariableInit(VariableInit),
    VariableInitDestruct(VariableInitDestruct),
    VariableSet(VariableSet),
    VariableSetDestruct(VariableSetDestruct),
    IfCondition(IfCondition),
    IfChain(IfChain),
    ShorthandAdd(ShorthandAdd),
    ShorthandSub(ShorthandSub),
    ShorthandMul(ShorthandMul),
    ShorthandDiv(ShorthandDiv),
    ShorthandModulo(ShorthandModulo),
    InfiniteLoop(InfiniteLoop),
    IterLoop(IterLoop),
    WhileLoop(WhileLoop),
    Break(Break),
    Continue(Continue),
    FunctionDeclaration(FunctionDeclaration),
    Return(Return),
    Fail(Fail),
    Import(Import),
    Main(Main),
    Test(Test),
    Cd(Cd),
    Echo(Echo),
    Mv(Mv),
    Touch(Touch),
    Exit(Exit),
    Command(Command),
    CommandModifier(CommandModifier),
    Comment(Comment),
    CommentDoc(CommentDoc),
    Sleep(Sleep),
    Rm(Rm),
    Clear(Clear),
    Await(Await),
    Cp(Cp),
    Disown(Disown),
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub value: Option<StmtType>,
}

impl Statement {
    pub fn get_docs_item_name(&self) -> Option<String> {
        match &self.value {
            Some(StmtType::FunctionDeclaration(inner)) => Some(inner.name.clone()),
            _ => None,
        }
    }
}

impl SyntaxModule<ParserMetadata> for Statement {
    syntax_name!("Statement");

    fn new() -> Self {
        Statement { value: None }
    }

    #[allow(unused_assignments)]
    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        // Order matters here
        parse_statement!(
            [
                // Imports
                Import,
                // Functions
                FunctionDeclaration,
                Main,
                Test,
                Return,
                Fail,
                // Loops
                InfiniteLoop,
                IterLoop,
                WhileLoop,
                Break,
                Continue,
                // Conditions
                IfChain,
                IfCondition,
                // Command
                Echo,
                Mv,
                Cd,
                Exit,
                CommandModifier,
                Command,
                Sleep,
                Rm,
                Clear,
                Await,
                Cp,
                Touch,
                Disown,
                // Variables
                VariableInitDestruct,
                VariableSetDestruct,
                VariableInit,
                VariableSet,
                // Short hand
                ShorthandAdd,
                ShorthandSub,
                ShorthandMul,
                ShorthandDiv,
                ShorthandModulo,
                // Comment doc
                CommentDoc,
                Comment,
                // Expression
                Expr
            ],
            |module, cons| {
                match syntax(meta, &mut module) {
                    Ok(()) => {
                        self.value = Some(cons(module));
                        Ok(())
                    }
                    Err(details) => Err(details),
                }
            }
        )
    }
}

impl TypeCheckModule for Statement {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.value.as_mut().unwrap().typecheck(meta)
    }
}

impl TranslateModule for Statement {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        self.value.as_ref().unwrap().translate(meta)
    }
}

impl DocumentationModule for Statement {
    fn document(&self, meta: &ParserMetadata) -> String {
        self.value.as_ref().unwrap().document(meta)
    }
}
