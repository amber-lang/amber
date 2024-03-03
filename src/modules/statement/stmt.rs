use heraclitus_compiler::prelude::*;
use itertools::Itertools;
use crate::docs::module::DocumentationModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use crate::modules::expression::expr::Expr;
use crate::translate::module::TranslateModule;
use crate::modules::variable::{
    init::VariableInit,
    set::VariableSet
};
use crate::modules::command::{
    statement::CommandStatement,
    modifier::CommandModifier
};
use crate::handle_types;
use crate::modules::condition::{
    ifchain::IfChain,
    ifcond::IfCondition
};
use crate::modules::shorthand::{
    add::ShorthandAdd,
    sub::ShorthandSub,
    mul::ShorthandMul,
    div::ShorthandDiv,
    modulo::ShorthandModulo
};
use crate::modules::loops::{
    infinite_loop::InfiniteLoop,
    iter_loop::IterLoop,
    break_stmt::Break,
    continue_stmt::Continue
};
use crate::modules::function::{
    declaration::FunctionDeclaration,
    ret::Return,
    fail::Fail
};
use crate::modules::imports::import::Import;
use crate::modules::main::Main;
use crate::modules::builtin::echo::Echo;
use super::comment_doc::CommentDoc;

#[derive(Debug, Clone)]
pub enum StatementType {
    Expr(Expr),
    VariableInit(VariableInit),
    VariableSet(VariableSet),
    CommandStatement(CommandStatement),
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
    Echo(Echo),
    CommandModifier(CommandModifier),
    CommentDoc(CommentDoc)
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
        CommandModifier, CommandStatement, Echo,
        // Comment doc
        CommentDoc,
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
            // Handle comments
            if let Some(token) = meta.get_current_token() {
                if token.word.starts_with("//") && !token.word.starts_with("///") {
                    meta.increment_index();
                    continue
                }
            }
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
        // Translate the statement
        let translated = self.translate_match(meta, self.value.as_ref().unwrap());
        // This is a workaround that handles $(...) which cannot be used as a statement
        let translated = (matches!(self.value, Some(StatementType::Expr(_))) || translated.starts_with("$(") || translated.starts_with("\"$("))
            .then(|| format!("echo {} > /dev/null 2>&1", translated))
            .unwrap_or(translated);
        // Get all the required supplemental statements
        let indentation = meta.gen_indent();
        let statements = meta.stmt_queue.drain(..).map(|st| indentation.clone() + &st + ";\n").join("");
        // Return all the statements
        statements + &indentation + &translated
    }
}

impl DocumentationModule for Statement {
    fn document(&self) -> String {
        // Document the statement
        let documented = self.document_match(self.value.as_ref().unwrap());
        documented
    }
}
