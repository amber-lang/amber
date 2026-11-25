use super::comment::Comment;
use super::comment_doc::CommentDoc;
use crate::docs::module::DocumentationModule;
use crate::modules::builtin::{
    cd::Cd, clear::Clear, cp::Cp, disown::Disown, echo::Echo, exit::Exit, lock::Lock, mv::Mv,
    rm::Rm, sleep::Sleep, touch::Touch, wait::Await,
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
    Lock(Box<Lock>),
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
                Lock,
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

impl Statement {
    pub fn terminates_control_flow(&self) -> bool {
        match &self.value {
            Some(StmtType::Return(_)) => true,
            Some(StmtType::Exit(_)) => true,
            Some(StmtType::Fail(_)) => true,
            Some(StmtType::Continue(_)) => true,
            Some(StmtType::IfCondition(if_cond)) => if_cond.terminates_control_flow(),
            Some(StmtType::IfChain(if_chain)) => if_chain.terminates_control_flow(),
            Some(StmtType::CommandModifier(cmd_mod)) => cmd_mod
                .block
                .as_ref()
                .is_some_and(|b| b.terminates_control_flow()),
            Some(StmtType::Command(cmd)) => cmd.terminates_control_flow(),
            Some(StmtType::InfiniteLoop(inf_loop)) => inf_loop.terminates_control_flow(),
            _ => false,
        }
    }

    pub fn has_reachable_break(&self) -> bool {
        match &self.value {
            Some(StmtType::Break(_)) => true,
            Some(StmtType::Expr(expr)) => expr.has_reachable_break(),
            Some(StmtType::VariableInit(v)) => v.has_reachable_break(),
            Some(StmtType::VariableInitDestruct(v)) => v.has_reachable_break(),
            Some(StmtType::VariableSet(v)) => v.has_reachable_break(),
            Some(StmtType::VariableSetDestruct(v)) => v.has_reachable_break(),
            Some(StmtType::ShorthandAdd(v)) => v.has_reachable_break(),
            Some(StmtType::ShorthandSub(v)) => v.has_reachable_break(),
            Some(StmtType::ShorthandMul(v)) => v.has_reachable_break(),
            Some(StmtType::ShorthandDiv(v)) => v.has_reachable_break(),
            Some(StmtType::ShorthandModulo(v)) => v.has_reachable_break(),
            Some(StmtType::IfCondition(v)) => v.has_reachable_break(),
            Some(StmtType::IfChain(v)) => v.has_reachable_break(),
            Some(StmtType::IterLoop(v)) => v.iter_expr.has_reachable_break(),
            Some(StmtType::CommandModifier(v)) => v.block.has_reachable_break(),
            Some(StmtType::Command(v)) => v.has_reachable_break(),
            Some(StmtType::Return(v)) => v.expr.has_reachable_break(),
            Some(StmtType::Fail(v)) => v.expr.has_reachable_break(),
            Some(StmtType::Cd(v)) => v.has_reachable_break(),
            Some(StmtType::Echo(v)) => v.has_reachable_break(),
            Some(StmtType::Mv(v)) => v.has_reachable_break(),
            Some(StmtType::Touch(v)) => v.has_reachable_break(),
            Some(StmtType::Exit(v)) => v.has_reachable_break(),
            Some(StmtType::Sleep(v)) => v.has_reachable_break(),
            Some(StmtType::Lock(v)) => v.has_reachable_break(),
            Some(StmtType::Rm(v)) => v.has_reachable_break(),
            Some(StmtType::Await(v)) => v.has_reachable_break(),
            Some(StmtType::Cp(v)) => v.has_reachable_break(),
            // Nested InfiniteLoop / WhileLoop and unexecuted declarations isolate their own `break`.
            _ => false,
        }
    }
}
