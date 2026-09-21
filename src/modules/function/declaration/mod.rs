pub mod check;
pub mod docs;
pub mod parse;

use self::check::{
    handle_existing_function, register_function, validate_failability, validate_optional_params,
    validate_unique_param_names,
};
use self::parse::{
    is_functions_comment_doc, parse_parameters, parse_return_type, scan_body_for_failure,
};
use super::core::signature::{FunctionDeclId, FunctionParam, FunctionSignature};
use crate::docs::module::DocumentationModule;
use crate::modules::block::Block;
use crate::modules::function::core::signature::{FunctionFragmentSignature, FunctionVariantParam};
use crate::modules::prelude::*;
use crate::modules::statement::comment_doc::CommentDoc;
use crate::modules::typecheck::TypeCheckModule;
use crate::modules::types::Type;
use crate::modules::variable::variable_name_extensions;
use crate::raw_fragment;
use crate::utils::cc_flags::{get_ccflag_by_name, CCFlags};
use crate::utils::context::Context;
use crate::utils::ParserMetadata;
use amber_meta::AutoKeyword;
use heraclitus_compiler::prelude::*;
use itertools::{izip, Itertools};
use std::collections::HashSet;

#[derive(Debug, Clone, AutoKeyword)]
#[keyword = "ref"]
#[kind = "stmt"]
#[allow(dead_code)]
pub struct Ref;

#[derive(Debug, Clone, AutoKeyword)]
#[keyword = "fun"]
#[kind = "stmt"]
pub struct FunctionDeclaration {
    pub name: String,
    pub params: Vec<FunctionParam>,
    pub returns: Type,
    pub id: FunctionDeclId,
    pub is_public: bool,
    pub flags: HashSet<CCFlags>,
    pub comment: Option<CommentDoc>,
    /// Function signature prepared for docs generation
    pub doc_signature: Option<String>,
    /// Function body context for typecheck phase
    pub function_body: Option<Block>,
    /// Whether the body can fail, and therefore whether callers must handle it
    pub is_failable: bool,
    /// Token for function name (for error positioning)
    pub name_token: Option<Token>,
}

impl SyntaxModule<ParserMetadata> for FunctionDeclaration {
    syntax_name!("Function Declaration");

    fn new() -> Self {
        FunctionDeclaration {
            name: String::new(),
            params: vec![],
            returns: Type::Generic,
            flags: HashSet::new(),
            id: FunctionDeclId::new(0),
            is_public: false,
            comment: None,
            doc_signature: None,
            function_body: None,
            is_failable: false,
            name_token: None,
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        // Parse the function comment
        if is_functions_comment_doc(meta) {
            let mut comment = CommentDoc::new();
            syntax(meta, &mut comment)?;
            self.comment = Some(comment);
        }
        // Get all the user-defined compiler flags
        while let Ok(flag) = token_by(meta, |val| val.starts_with("#[")) {
            // Push to the flags vector as it is more safe in case of parsing errors
            self.flags
                .insert(get_ccflag_by_name(&flag[2..flag.len() - 1]));
        }
        let doc_index = meta.get_index();
        // Check if this function is public
        if token(meta, "pub").is_ok() {
            self.is_public = true;
        }
        token(meta, "fun")?;
        // Get the function name
        self.name_token = meta.get_current_token();
        self.name = variable(meta, variable_name_extensions())?;
        context!(
            {
                self.params = parse_parameters(meta)?;
                let returns = parse_return_type(meta)?;
                self.returns = returns.kind.clone();

                // Look ahead through the body to learn whether it can fail,
                // then rewind to the start
                let start_pos = meta.get_index();
                token(meta, "{")?;
                self.is_failable = scan_body_for_failure(meta, start_pos);

                // Without a declared return type there is nowhere to put a '?',
                // so the marker is simply inferred from the body.
                let declared_failable = if returns.kind == Type::Generic {
                    self.is_failable
                } else {
                    returns.declared_failable
                };
                validate_failability(meta, self.is_failable, declared_failable, &returns)?;

                // Store function body for typecheck phase
                let mut block = Block::new().with_condition();
                let was_fun_ctx = meta.context.is_fun_ctx;
                meta.context.is_fun_ctx = true;
                let result =
                    meta.with_context_fn(Context::set_cc_flags, self.flags.clone(), |meta| {
                        syntax(meta, &mut block)
                    });
                meta.context.is_fun_ctx = was_fun_ctx;
                result?;
                self.function_body = Some(block);

                self.doc_signature = Some(self.render_function_signature(meta, doc_index)?);
                Ok(())
            },
            |pos| {
                error_pos!(
                    meta,
                    pos,
                    format!("Failed to parse function declaration '{}'", self.name)
                )
            }
        )
    }
}

impl TypeCheckModule for FunctionDeclaration {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        // Check if we are in the global scope
        if !meta.is_global_scope() {
            return error!(
                meta,
                self.name_token.clone(),
                "Functions can only be declared in the global scope"
            );
        }

        // Check if function already exists
        handle_existing_function(meta, self.name_token.clone())?;

        meta.with_context_fn(Context::set_cc_flags, self.flags.clone(), |meta| {
            validate_unique_param_names(meta, &self.params)?;
            validate_optional_params(meta, &mut self.params)?;

            // Create function context and add to memory
            let block = self.function_body.clone().unwrap_or_else(Block::new);
            let mut ctx = meta.context.clone();
            ctx.is_fun_ctx = true;
            ctx.expr.clear();

            let signature = FunctionSignature {
                id: meta.gen_fun_id(),
                name: self.name.clone(),
                params: self.params.clone(),
                returns: self.returns.clone(),
                is_public: self.is_public,
                is_failable: self.is_failable,
            };
            self.id = signature.id;
            register_function(meta, self.name_token.clone(), signature, ctx, block)
        })
    }
}

impl TranslateModule for FunctionDeclaration {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let mut result = vec![];
        let variants = meta.fun_cache.get_variants_cloned(self.id).unwrap();
        let prev_fun_meta = meta.fun_frag_sig.clone();
        // Translate each one of them
        for variant in variants.iter() {
            meta.fun_frag_sig = Some(FunctionFragmentSignature {
                name: self.name.clone(),
                declaration_id: self.id,
                variant_id: variant.id,
                return_type: variant.returns.clone(),
            });

            // Document in code function variant's signature
            let argument_types = izip!(self.params.iter(), variant.param_types.iter())
                .map(|(param, ty)| format!("{}: {}", param.name, ty))
                .join(", ");
            result.push(raw_fragment!("# {}({argument_types})", self.name));

            // Prepare arguments
            let params = izip!(
                &self.params,
                &variant.param_global_ids,
                &variant.param_types
            )
            .map(FunctionVariantParam::from_tuple)
            .collect::<Vec<_>>();

            let body = variant.body.translate(meta);
            result.push(
                FunctionDeclFragment::new(
                    &self.name,
                    self.id,
                    variant.id,
                    body,
                    &params,
                    meta.target.shell,
                )
                .to_frag(),
            );
        }
        // Restore the function name
        meta.fun_frag_sig = prev_fun_meta;
        // Return the translation
        BlockFragment::new(result, false).to_frag()
    }
}

impl DocumentationModule for FunctionDeclaration {
    fn document(&self, meta: &ParserMetadata) -> String {
        let mut result = vec![];
        result.push(format!("## `{}`\n", self.name));
        result.push("```ab".to_string());
        result.push(self.doc_signature.to_owned().unwrap());
        result.push("```\n".to_string());
        if let Some(comment) = &self.comment {
            let comment_text = comment.document(meta);
            // Check if comment has Usage section with code block and insert import statement
            let comment_text = self.insert_usage_import_statement(meta, comment_text);
            result.push(comment_text);
        }
        result.push("".to_string());
        result.join("\n")
    }
}
