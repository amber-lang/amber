use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use crate::modules::expression::expr::Expr;
use crate::translate::module::TranslateModule;
use crate::modules::variable::{
    init::VariableInit,
    set::VariableSet,
};
use crate::modules::command::modifier::CommandModifier;
use crate::modules::command::cmd::Command;
use crate::{
    parse_statement, typecheck_statement, translate_statement, document_statement
};
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
use crate::modules::test::Test;
use crate::modules::builtin::{
    echo::Echo,
    mv::Mv,
    cd::Cd,
    exit::Exit,
    touch::Touch
};
use super::comment_doc::CommentDoc;
use super::comment::Comment;

#[derive(Debug, Clone)]
pub enum StmtType {
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
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub value: Option<StmtType>
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
        Statement {
            value: None
        }
    }

    #[allow(unused_assignments)]
    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        // Order matters here
        parse_statement!([
            // Imports
            Import,
            // Functions
            FunctionDeclaration, Main, Test, Return, Fail,
            // Loops
            InfiniteLoop, IterLoop, WhileLoop, Break, Continue,
            // Conditions
            IfChain, IfCondition,
            // Command
            CommandModifier, Echo, Mv, Cd, Exit, Command,
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
        ], |module, cons| {
            match syntax(meta, &mut module) {
                Ok(()) => {
                    self.value = Some(cons(module));
                    Ok(())
                }
                Err(details) => Err(details)
            }
        })
    }
}

impl TypeCheckModule for Statement {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        typecheck_statement!(meta, self.value.as_mut().unwrap(), [
            Break, Cd, Command, CommandModifier, Comment, CommentDoc, Continue, Echo,
            Exit, Expr, Fail, FunctionDeclaration, IfChain, IfCondition,
            Import, InfiniteLoop, IterLoop, Main, Mv, Return, ShorthandAdd,
            ShorthandDiv, ShorthandModulo, ShorthandMul, ShorthandSub,
            Test, VariableInit, VariableSet, WhileLoop
        ]);
        Ok(())
    }
}

impl TranslateModule for Statement {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        // Translate the staxtement
        let statement = self.value.as_ref().unwrap();
        // This is a workaround that handles $(...) which cannot be used as a statement
        translate_statement!(statement, [
            Import,
            FunctionDeclaration, Main, Test, Return, Fail,
            InfiniteLoop, IterLoop, WhileLoop, Break, Continue,
            IfChain, IfCondition,
            CommandModifier, Echo, Mv, Cd, Exit, Command,
            VariableInit, VariableSet,
            ShorthandAdd, ShorthandSub,
            ShorthandMul, ShorthandDiv,
            ShorthandModulo,
            CommentDoc, Comment,
            Expr
        ], |inner_module| {
            if let StmtType::Expr(_) = statement {
                inner_module.translate(meta);
                FragmentKind::Empty
            } else {
                inner_module.translate(meta)
            }
        })
    }
}

impl DocumentationModule for Statement {
    fn document(&self, meta: &ParserMetadata) -> String {
        // Document the statement
        let statement = self.value.as_ref().unwrap();
        document_statement!(statement, [
            Import,
            FunctionDeclaration, Main, Test, Return, Fail,
            InfiniteLoop, IterLoop, WhileLoop, Break, Continue,
            IfChain, IfCondition,
            CommandModifier, Echo, Mv, Cd, Exit, Command,
            VariableInit, VariableSet,
            ShorthandAdd, ShorthandSub,
            ShorthandMul, ShorthandDiv,
            ShorthandModulo,
            CommentDoc, Comment,
            Expr
        ], inner_module, inner_module.document(meta))
    }
}
