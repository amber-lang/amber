use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use crate::modules::expression::expr::{Expr, ExprType};
use crate::translate::module::TranslateModule;
use crate::modules::variable::{
    init::VariableInit,
    set::VariableSet,
};
use crate::modules::command::modifier::CommandModifier;
use crate::handle_types;
use crate::modules::condition::{
    ifchain::IfChain,
    ifcond::IfCondition,
};
use crate::modules::shorthand::{
    add::ShorthandAdd,
    sub::ShorthandSub,
    mul::ShorthandMul,
    div::ShorthandDiv,
    modulo::ShorthandModulo,
};
use crate::modules::loops::{
    infinite_loop::InfiniteLoop,
    iter_loop::IterLoop,
    while_loop::WhileLoop,
    break_stmt::Break,
    continue_stmt::Continue,
};
use crate::modules::function::{
    declaration::FunctionDeclaration,
    ret::Return,
    fail::Fail,
};
use crate::modules::imports::import::Import;
use crate::modules::main::Main;
use crate::modules::builtin::{
    echo::Echo,
    mv::Mv,
    cd::Cd,
    exit::Exit,
};
use super::comment_doc::CommentDoc;
use super::comment::Comment;

#[derive(Debug, Clone)]
pub enum StatementType {
    Expr(Expr),
    VariableInit(VariableInit),
    VariableSet(VariableSet),
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
    Cd(Cd),
    Echo(Echo),
    Mv(Mv),
    Exit(Exit),
    CommandModifier(CommandModifier),
    Comment(Comment),
    CommentDoc(CommentDoc),
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub value: Option<StatementType>
}

impl Statement {
    handle_types!(StatementType, [
        // Imports
        Import,
        // Functions
        FunctionDeclaration, Main, Return, Fail,
        // Loops
        InfiniteLoop, IterLoop, WhileLoop, Break, Continue,
        // Conditions
        IfChain, IfCondition,
        // Command
        CommandModifier, Echo, Mv, Cd, Exit,
        // Variables
        VariableInit, VariableSet,
        // Short hand
        ShorthandAdd, ShorthandSub,
        ShorthandMul, ShorthandDiv,
        ShorthandModulo,
        // Comment doc
        CommentDoc, Comment,
        // Expression
        Expr
    ]);

    // Get result out of the provided module and save it in the internal state
    fn get<M,S>(&mut self, meta: &mut M, mut module: S, cb: impl Fn(S) -> StatementType) -> SyntaxResult
    where
        M: Metadata,
        S: SyntaxModule<M>
    {
        match syntax(meta, &mut module) {
            Ok(()) => {
                self.value = Some(cb(module));
                Ok(())
            }
            Err(details) => Err(details)
        }
    }

    pub fn get_docs_item_name(&self) -> Option<String> {
        match &self.value {
            Some(StatementType::FunctionDeclaration(inner)) => Some(inner.name.clone()),
            _ => None,
        }
    }
}

impl SyntaxModule<ParserMetadata> for Statement {
    syntax_name!("Statement");

    fn new() -> Self {
        Statement {
            value: None
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let mut error = None;
        let statements = self.get_modules();
        for statement in statements {
            // Try to parse the statement
            match self.parse_match(meta, statement) {
                Ok(()) => return Ok(()),
                Err(failure) => {
                    match failure {
                        Failure::Loud(err) => return Err(Failure::Loud(err)),
                        Failure::Quiet(err) => error = Some(err)
                    }
                }
            }
        }
        Err(Failure::Quiet(error.unwrap()))
    }
}

impl TypeCheckModule for Statement {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        if let Some(statement) = &mut self.value {
            match statement {
                StatementType::Expr(expr) => expr.typecheck(meta)?,
                StatementType::VariableInit(var_init) => var_init.typecheck(meta)?,
                StatementType::VariableSet(var_set) => var_set.typecheck(meta)?,
                StatementType::IfCondition(if_cond) => if_cond.typecheck(meta)?,
                StatementType::IfChain(if_chain) => if_chain.typecheck(meta)?,
                StatementType::ShorthandAdd(shorthand) => shorthand.typecheck(meta)?,
                StatementType::ShorthandSub(shorthand) => shorthand.typecheck(meta)?,
                StatementType::ShorthandMul(shorthand) => shorthand.typecheck(meta)?,
                StatementType::ShorthandDiv(shorthand) => shorthand.typecheck(meta)?,
                StatementType::ShorthandModulo(shorthand) => shorthand.typecheck(meta)?,
                StatementType::InfiniteLoop(loop_stmt) => loop_stmt.typecheck(meta)?,
                StatementType::IterLoop(iter_loop) => iter_loop.typecheck(meta)?,
                StatementType::WhileLoop(while_loop) => while_loop.typecheck(meta)?,
                StatementType::Break(_) => {}, // No type checking needed for break
                StatementType::Continue(_) => {}, // No type checking needed for continue
                StatementType::FunctionDeclaration(func_decl) => func_decl.typecheck(meta)?,
                StatementType::Return(ret) => ret.typecheck(meta)?,
                StatementType::Fail(fail) => fail.typecheck(meta)?,
                StatementType::Import(_) => {}, // No type checking needed for imports
                StatementType::Main(main) => main.typecheck(meta)?,
                StatementType::Cd(cd) => cd.typecheck(meta)?,
                StatementType::Echo(echo) => echo.typecheck(meta)?,
                StatementType::Mv(mv) => mv.typecheck(meta)?,
                StatementType::Exit(exit) => exit.typecheck(meta)?,
                StatementType::CommandModifier(cmd_mod) => cmd_mod.typecheck(meta)?,
                StatementType::Comment(_) => {}, // No type checking needed for comments
                StatementType::CommentDoc(_) => {}, // No type checking needed for doc comments
            }
        }
        Ok(())
    }
}

impl TranslateModule for Statement {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        // Translate the staxtement
        let statement = self.value.as_ref().unwrap();
        // This is a workaround that handles $(...) which cannot be used as a statement
        match statement {
            StatementType::Expr(expr) => {
                match &expr.value {
                    Some(ExprType::Command(cmd)) => {
                        cmd.translate_command_statement(meta)
                    },
                    _ => {
                        self.translate_match(meta, statement);
                        FragmentKind::Empty
                    }
                }
            },
            _ => {
                self.translate_match(meta, statement)
            }
        }
    }
}

impl DocumentationModule for Statement {
    fn document(&self, meta: &ParserMetadata) -> String {
        // Document the statement
        let documented = self.document_match(meta, self.value.as_ref().unwrap());
        documented
    }
}
