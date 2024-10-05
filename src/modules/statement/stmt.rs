use heraclitus_compiler::prelude::*;
use itertools::Itertools;
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
        InfiniteLoop, IterLoop, Break, Continue,
        // Conditions
        IfChain, IfCondition,
        // Variables
        VariableInit, VariableSet,
        // Short hand
        ShorthandAdd, ShorthandSub,
        ShorthandMul, ShorthandDiv,
        ShorthandModulo,
        // Command
        CommandModifier, Echo, Mv, Cd, Exit,
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

impl TranslateModule for Statement {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        // Translate the staxtement
        let statement = self.value.as_ref().unwrap();
        // This is a workaround that handles $(...) which cannot be used as a statement
        let translated = match statement {
            StatementType::Expr(expr) => match &expr.value {
                Some(ExprType::Command(cmd)) => cmd.translate_command_statement(meta),
                _ => format!("echo {} > /dev/null 2>&1", self.translate_match(meta, statement))
            },
            _ => self.translate_match(meta, statement)
        };
        // Get all the required supplemental statements
        let indentation = meta.gen_indent();
        let statements = meta.stmt_queue.drain(..).map(|st| indentation.clone() + st.trim_end_matches(';') + ";\n").join("");
        // Return all the statements
        statements + &indentation + &translated
    }
}

impl DocumentationModule for Statement {
    fn document(&self, meta: &ParserMetadata) -> String {
        // Document the statement
        let documented = self.document_match(meta, self.value.as_ref().unwrap());
        documented
    }
}
