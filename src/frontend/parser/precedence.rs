use crate::frontend::located::Located;

use super::internal_ast::{ASTBinOp, ASTExpression};

impl Located<ASTExpression> {
    pub fn add_using_precedence(self, op2: ASTBinOp, other: Located<ASTExpression>) -> Located<ASTExpression> {
        let loc1 = self.location();
        let loc2 = other.location();
        let v1 = self.value;
        let v2 = other.value;

        let loc_overall = loc1.merge(loc2).location();
        let v_overall = match v1 {
            ASTExpression::StringLiteral { .. } | ASTExpression::IntegerLiteral { .. }  |
            ASTExpression::Call { .. } | ASTExpression::UOp { .. } |
            ASTExpression::Invalid(..)
            => {
                ASTExpression::BinOp { arg1: Box::new(loc1.replace(v1)), op: op2, arg2: Box::new(loc2.replace(v2)) }
            }
            ASTExpression::BinOp { arg1, op: op1, arg2 } =>  {
                if op1.tighter_than(&op2) {
                    ASTExpression::BinOp { arg1, op: op1, arg2: Box::new(arg2.add_using_precedence(op2, loc2.replace(v2))) }
                } else {
                    ASTExpression::BinOp { 
                        arg1: Box::new(loc1.replace(ASTExpression::BinOp { arg1, op: op1, arg2})),
                        op: op2,
                        arg2: Box::new(loc2.replace(v2))
                    }
                }

            }
        };
        loc_overall.replace(v_overall)
    }
}

impl ASTBinOp {
    fn tighter_than(&self, other: &ASTBinOp) -> bool {
        if self.precedence_level() < other.precedence_level() {
            return true
        } else if self.precedence_level() == other.precedence_level() {
            // i'm on the left!!!
            return self.left_associative();
        } else {
            return false;
        }
    }

    fn precedence_level(&self) -> usize {
        match self {
            ASTBinOp::Multiply | ASTBinOp::Divide => 0,
            ASTBinOp::Add | &ASTBinOp::Subtract => 1,
        }
    }

    fn left_associative(&self) -> bool {
        match self {
            ASTBinOp::Add => true,
            ASTBinOp::Subtract => true,
            ASTBinOp::Multiply => true,
            ASTBinOp::Divide => true,
        }
    }
}