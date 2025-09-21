use std::collections::HashSet;
use crate::raw_fragment;
use std::{env, fs};
use std::ffi::OsStr;
use std::path::Path;

use crate::fragments;
use crate::modules::prelude::*;
use heraclitus_compiler::prelude::*;
use itertools::izip;
use crate::modules::statement::comment_doc::CommentDoc;
use crate::modules::expression::expr::Expr;
use crate::modules::types::{Type, Typed};
use crate::modules::variable::variable_name_extensions;
use crate::utils::cc_flags::get_ccflag_by_name;
use crate::utils::context::Context;
use crate::utils::function_cache::FunctionInstance;
use crate::utils::function_interface::FunctionInterface;
use crate::modules::types::parse_type;
use crate::utils::function_metadata::FunctionMetadata;
use super::declaration_utils::*;

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub name: String,
    pub arg_refs: Vec<bool>,
    pub arg_names: Vec<String>,
    pub arg_types: Vec<Type>,
    pub arg_optionals: Vec<Expr>,
    pub returns: Type,
    pub id: usize,
    pub is_public: bool,
    pub comment: Option<CommentDoc>,
    /// Function signature prepared for docs generation
    pub doc_signature: Option<String>
}

impl FunctionDeclaration {
    fn set_args_as_variables(&self, _meta: &mut TranslateMetadata, function: &FunctionInstance, arg_refs: &[bool]) -> Option<FragmentKind> {
        if !self.arg_names.is_empty() {
            let mut result = vec![];
            for (index, (name, kind, is_ref)) in izip!(self.arg_names.clone(), &function.args, arg_refs).enumerate() {
                match (is_ref, kind) {
                    (false, Type::Array(_)) => result.push(raw_fragment!("local {name}=(\"${{!{}}}\")", index + 1)),
                    _ => result.push(raw_fragment!("local {name}=${}", index + 1)),
                }
            }
            Some(BlockFragment::new(result, true).to_frag())
        } else { None }
    }

    fn get_space(&self, parentheses: usize, before: &str, word: &str) -> String {
        if parentheses == 0 && word == "("
            || word == ":"
            || word == ")"
            || word == "]"
            || word == ","
            || before == "["
            || before == "("
        {
            return String::new()
        }
        " ".to_string()
    }

    fn render_function_signature(&self, meta: &ParserMetadata, doc_index: usize) -> Result<String, Failure> {
        let mut result = String::new();
        let mut index = doc_index;
        let mut parentheses = 0;
        let mut before = String::new();
        loop {
            let cur_token = meta.context.expr.get(index);
            let cur_word = cur_token.map_or_else(String::new, |v| v.word.clone());
            if !result.is_empty() {
                result.push_str(&self.get_space(parentheses, &before, &cur_word))
            }
            before.clone_from(&cur_word);
            match cur_word.as_str() {
                "(" => parentheses += 1,
                ")" => parentheses -= 1,
                "{" if parentheses == 0 => break,
                "" => {
                    return error!(meta, cur_token.cloned(), "Error when parsing function signature. Please report this issue.");
                }
                _ => {}
            }
            result.push_str(&cur_word);
            index += 1;
        }
        Ok(result)
    }
}

impl SyntaxModule<ParserMetadata> for FunctionDeclaration {
    syntax_name!("Function Declaration");

    fn new() -> Self {
        FunctionDeclaration {
            name: String::new(),
            arg_names: vec![],
            arg_types: vec![],
            arg_refs: vec![],
            arg_optionals: vec![],
            returns: Type::Generic,
            id: 0,
            is_public: false,
            comment: None,
            doc_signature: None,
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        // Parse the function comment
        if is_functions_comment_doc(meta) {
            let mut comment = CommentDoc::new();
            syntax(meta, &mut comment)?;
            self.comment = Some(comment);
        }
        let mut flags = HashSet::new();
        // Get all the user-defined compiler flags
        while let Ok(flag) = token_by(meta, |val| val.starts_with("#[")) {
            // Push to the flags vector as it is more safe in case of parsing errors
            flags.insert(get_ccflag_by_name(&flag[2..flag.len() - 1]));
        }
        let tok = meta.get_current_token();
        let doc_index = meta.get_index();
        // Check if this function is public
        if token(meta, "pub").is_ok() {
            self.is_public = true;
        }
        if let Err(err) = token(meta, "fun") {
            if !flags.is_empty() {
                return error!(meta, tok, "Compiler flags can only be used in function declarations")
            }
            return Err(err)
        }
        // Check if we are in the global scope
        if !meta.is_global_scope() {
            return error!(meta, tok, "Functions can only be declared in the global scope")
        }
        // Get the function name
        let tok = meta.get_current_token();
        self.name = variable(meta, variable_name_extensions())?;
        handle_existing_function(meta, tok.clone())?;
        let mut optional = false;
        context!({
            // Set the compiler flags
            meta.with_context_fn(Context::set_cc_flags, flags, |meta| {
                // Get the arguments
                token(meta, "(")?;
                let mut seen_argument_names = HashSet::new();
                loop {
                    if token(meta, ")").is_ok() {
                        break
                    }
                    let is_ref = token(meta, "ref").is_ok();
                    let name_token = meta.get_current_token();
                    let name = variable(meta, variable_name_extensions())?;

                    // Check for duplicate argument name
                    if !seen_argument_names.insert(name.clone()) {
                        return error!(meta, name_token, format!("Argument '{name}' is already defined"));
                    }

                    // Optionally parse the argument type
                    let mut arg_type = Type::Generic;
                    match token(meta, ":") {
                        Ok(_) => {
                            self.arg_refs.push(is_ref);
                            self.arg_names.push(name.clone());
                            arg_type = parse_type(meta)?;
                            self.arg_types.push(arg_type.clone());
                        },
                        Err(_) => {
                            self.arg_refs.push(is_ref);
                            self.arg_names.push(name.clone());
                            self.arg_types.push(Type::Generic);
                        }
                    }
                    match token(meta, "=") {
                        Ok(_) => {
                            if is_ref {
                                return error!(meta, name_token, "A ref cannot be optional");
                            }
                            optional = true;
                            let mut expr = Expr::new();
                            syntax(meta, &mut expr)?;
                            if !expr.get_type().is_allowed_in(&arg_type) {
                                return error!(meta, name_token, "Optional argument does not match annotated type");
                            }
                            self.arg_optionals.push(expr);
                        },
                        Err(_) => {
                            if optional {
                               return error!(meta, name_token, "All arguments following an optional argument must also be optional");
                            }
                        },
                    }
                    match token(meta, ")") {
                        Ok(_) => break,
                        Err(_) => token(meta, ",")?
                    };
                }
                let mut returns_tok = None;
                let mut declared_failable = false;
                // Optionally parse the return type
                match token(meta, ":") {
                    Ok(_) => {
                        returns_tok = meta.get_current_token();
                        self.returns = parse_type(meta)?;
                        if token(meta, "?").is_ok() {
                            declared_failable = true;
                        }
                    },
                    Err(_) => self.returns = Type::Generic
                }
                // Parse the body
                token(meta, "{")?;
                let (index_begin, index_end, is_failable) = skip_function_body(meta);
                if self.returns == Type::Generic {
                    declared_failable = is_failable;
                }
                if is_failable && !declared_failable {
                    return error!(meta, returns_tok, "Failable functions must have a '?' after the type name");
                }
                if !is_failable && declared_failable {
                    return error!(meta, returns_tok, "Infallible functions must not have a '?' after the type name");
                }

                // Create a new context with the function body
                let expr = meta.context.expr[index_begin..index_end].to_vec();
                let ctx = meta.context.clone().function_invocation(expr);
                token(meta, "}")?;
                self.doc_signature = Some(self.render_function_signature(meta, doc_index)?);
                // Add the function to the memory
                self.id = handle_add_function(meta, tok.clone(), FunctionInterface {
                    id: None,
                    name: self.name.clone(),
                    arg_names: self.arg_names.clone(),
                    arg_types: self.arg_types.clone(),
                    arg_refs: self.arg_refs.clone(),
                    returns: self.returns.clone(),
                    arg_optionals: self.arg_optionals.clone(),
                    is_public: self.is_public,
                    is_failable
                }, ctx)?;
                Ok(())
            })?;
            Ok(())
        }, |pos| {
            error_pos!(meta, pos, format!("Failed to parse function declaration '{}'", self.name))
        })
    }
}

impl TranslateModule for FunctionDeclaration {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let mut result = vec![];
        let blocks = meta.fun_cache.get_instances_cloned(self.id).unwrap();
        let prev_fun_meta = meta.fun_meta.clone();
        // Translate each one of them
        for (index, function) in blocks.iter().enumerate() {
            meta.fun_meta = Some(FunctionMetadata::new(&self.name, self.id, index, &self.returns));
            // Parse the function body
            let name = raw_fragment!("{}__{}_v{}", self.name, self.id, index);
            result.push(fragments!(name, "() {"));
            if let Some(args) = self.set_args_as_variables(meta, function, &self.arg_refs) {
                result.push(args);
            }
            result.push(function.block.translate(meta));
            result.push(fragments!("}\n"));
        }
        // Restore the function name
        meta.fun_meta = prev_fun_meta;
        // Return the translation
        BlockFragment::new(result, false).to_frag()
    }
}

impl DocumentationModule for FunctionDeclaration {
    fn document(&self, meta: &ParserMetadata) -> String {
        let mut result = vec![];
        result.push(format!("## `{}`\n", self.name));
        let references = self.create_test_references(meta, &mut result);
        result.push("```ab".to_string());
        result.push(self.doc_signature.to_owned().unwrap());
        result.push("```\n".to_string());
        if let Some(comment) = &self.comment {
            result.push(comment.document(meta));
        }
        if let Some(references) = references {
            for reference in references {
                result.push(reference);
            }
            result.push("".to_string());
        }
        result.push("".to_string());
        result.join("\n")
    }
}

impl FunctionDeclaration {
    fn create_test_references(&self, meta: &ParserMetadata, result: &mut Vec<String>) -> Option<Vec<String>> {
        if meta.doc_usage {
            let mut references = Vec::new();
            let exe_path = env::current_exe()
                .expect("Executable path not found");
            let root_path = exe_path.parent()
                .and_then(Path::parent)
                .and_then(Path::parent)
                .expect("Root path not found");
            let test_path = root_path.join("src")
                .join("tests")
                .join("stdlib");
            let lib_name = meta.context.path.as_ref()
                .map(Path::new)
                .and_then(Path::file_name)
                .and_then(OsStr::to_str)
                .map(|s| s.trim_end_matches(".ab"))
                .map(String::from)
                .unwrap_or_default();
            result.push(String::from("```ab"));
            result.push(format!("import {{ {} }} from \"std/{}\"", self.name, lib_name));
            result.push(String::from("```\n"));
            if test_path.exists() && test_path.is_dir() {
                if let Ok(entries) = fs::read_dir(test_path) {
                    let pattern1 = {
                        let pattern = format!("{}*.ab", self.name);
                        glob::Pattern::new(&pattern).unwrap()
                    };
                    let pattern2 = {
                        let pattern = format!("{}_{}*.ab", lib_name, self.name);
                        glob::Pattern::new(&pattern).unwrap()
                    };
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(file_name) = path.file_name().and_then(OsStr::to_str) {
                            if pattern1.matches(file_name) || pattern2.matches(file_name) {
                                references.push(format!(
                                    "* [{}](https://github.com/amber-lang/amber/blob/{}/src/tests/stdlib/{})",
                                    file_name,
                                    env!("CARGO_PKG_VERSION"),
                                    file_name,
                                ));
                            }
                        }
                    }
                }
            }
            if !references.is_empty() {
                references.sort();
                references.insert(0, String::from("You can check the original tests for code examples:"));
                return Some(references);
            }
        }
        None
    }
}
