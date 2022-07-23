use heraclitus_compiler::prelude::*;

enum StatementType {}

struct Statement {
    value: Option<StatementType>
}

impl Statement {
    fn statement_types(&self) -> Vec<StatementType> {
        vec![]
    }
    
    fn parse_statement(&mut self, meta: &mut DefaultMetadata, statement: StatementType) -> SyntaxResult {
        match statement {}
    }
}

impl SyntaxModule<DefaultMetadata> for Statement {
    fn new() -> Self {
        Statement {
            value: None
        }
    }

    fn parse(&mut self, meta: &mut DefaultMetadata) -> SyntaxResult {
        let mut error = None;
        let statements = self.statement_types();
        for statement in statements {
            match self.parse_statement(meta, statement) {
                Ok(()) => return Ok(()),
                Err(details) => error = Some(details)
            }
        }
        Err(error.unwrap())
    }
}