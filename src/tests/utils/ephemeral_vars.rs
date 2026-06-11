//! Tests for optimizer/ephemeral_vars.rs

use crate::optimizer::ephemeral_vars::remove_ephemeral_variables;
use crate::translate::fragments::var_expr::VarExprFragment;
use crate::translate::fragments::var_stmt::VarStmtFragment;
use crate::translate::fragments::raw::RawFragment;
use crate::translate::fragments::block::BlockFragment;
use crate::modules::prelude::*;
use crate::modules::types::Type;

#[cfg(test)]
mod ephemeral_vars_tests {
    use super::*;

    #[test]
    fn test_remove_ephemeral_simple_chain() {
        // Test: (eph = 5; var = eph) -> (var = 5)
        let eph_stmt = FragmentKind::VarStmt(VarStmtFragment::new("eph", Type::Int, FragmentKind::Raw(RawFragment::new("5"))).with_ephemeral(true));
        let var_stmt = FragmentKind::VarStmt(VarStmtFragment::new("var", Type::Int, FragmentKind::VarExpr(VarExprFragment::new("eph", Type::Int))));
        
        let mut block = FragmentKind::Block(BlockFragment {
            statements: vec![eph_stmt, var_stmt],
            increase_indent: false,
            needs_noop: false,
            is_conditional: false,
        });
        
        remove_ephemeral_variables(&mut block);
        
        match &block {
            FragmentKind::Block(block) => {
                assert_eq!(block.statements.len(), 1, "Ephemeral variable should be removed");
            }
            other => panic!("Expected FragmentKind::Block after optimization, got {:?}", other),
        }
    }

    #[test]
    fn test_remove_ephemeral_transitive_chain() {
        // Test: (eph1 = 5; eph2 = eph1; var = eph2) -> (var = 5)
        let eph1_stmt = FragmentKind::VarStmt(VarStmtFragment::new("eph1", Type::Int, FragmentKind::Raw(RawFragment::new("5"))).with_ephemeral(true));
        let eph2_stmt = FragmentKind::VarStmt(VarStmtFragment::new("eph2", Type::Int, FragmentKind::VarExpr(VarExprFragment::new("eph1", Type::Int))).with_ephemeral(true));
        let var_stmt = FragmentKind::VarStmt(VarStmtFragment::new("var", Type::Int, FragmentKind::VarExpr(VarExprFragment::new("eph2", Type::Int))));
        
        let mut block = FragmentKind::Block(BlockFragment {
            statements: vec![eph1_stmt, eph2_stmt, var_stmt],
            increase_indent: false,
            needs_noop: false,
            is_conditional: false,
        });
        
        remove_ephemeral_variables(&mut block);
        
        match &block {
            FragmentKind::Block(block) => {
                assert_eq!(block.statements.len(), 1, "All ephemeral variables should be removed");
            }
            other => panic!("Expected FragmentKind::Block after optimization, got {:?}", other),
        }
    }

    #[test]
    fn test_remove_ephemeral_preserves_regular_variable() {
        // Test: (regular = 5; var = regular) should NOT be optimized
        let regular_stmt = FragmentKind::VarStmt(VarStmtFragment::new("regular", Type::Int, FragmentKind::Raw(RawFragment::new("5"))));
        let var_stmt = FragmentKind::VarStmt(VarStmtFragment::new("var", Type::Int, FragmentKind::VarExpr(VarExprFragment::new("regular", Type::Int))));
        
        let mut block = FragmentKind::Block(BlockFragment {
            statements: vec![regular_stmt, var_stmt],
            increase_indent: false,
            needs_noop: false,
            is_conditional: false,
        });
        
        let original_len = match &block {
            FragmentKind::Block(b) => b.statements.len(),
            other => panic!("Expected FragmentKind::Block before optimization, got {:?}", other),
        };
        remove_ephemeral_variables(&mut block);
        
        match &block {
            FragmentKind::Block(block) => {
                assert_eq!(block.statements.len(), original_len, "Regular variables should not be removed");
            }
            other => panic!("Expected FragmentKind::Block after optimization, got {:?}", other),
        }
    }

    #[test]
    fn test_remove_ephemeral_no_change_single_statement() {
        // Test that a single statement is not modified
        let eph_stmt = FragmentKind::VarStmt(VarStmtFragment::new("eph", Type::Int, FragmentKind::Raw(RawFragment::new("5"))).with_ephemeral(true));
        
        let mut block = FragmentKind::Block(BlockFragment {
            statements: vec![eph_stmt],
            increase_indent: false,
            needs_noop: false,
            is_conditional: false,
        });
        
        let original_len = match &block {
            FragmentKind::Block(b) => b.statements.len(),
            other => panic!("Expected FragmentKind::Block before optimization, got {:?}", other),
        };
        remove_ephemeral_variables(&mut block);
        
        match &block {
            FragmentKind::Block(block) => {
                assert_eq!(block.statements.len(), original_len, "Single statement should not be modified");
            }
            other => panic!("Expected FragmentKind::Block after optimization, got {:?}", other),
        }
    }

    #[test]
    fn test_remove_ephemeral_empty_block() {
        // Test that an empty block is not modified
        let mut block = FragmentKind::Block(BlockFragment {
            statements: vec![],
            increase_indent: false,
            needs_noop: false,
            is_conditional: false,
        });
        
        remove_ephemeral_variables(&mut block);
        
        match &block {
            FragmentKind::Block(block) => {
                assert_eq!(block.statements.len(), 0, "Empty block should remain empty");
            }
            other => panic!("Expected FragmentKind::Block after optimization, got {:?}", other),
        }
    }
}
