// Reference tracking in expressions and statements
use super::super::types::SymbolInfo;
use super::DocumentStore;
use crate::ast::{Expression, Statement};
use std::collections::HashMap;

impl DocumentStore {
    pub(super) fn find_references_in_statements(
        &self,
        statements: &[Statement],
        symbols: &mut HashMap<String, SymbolInfo>,
    ) {
        for stmt in statements {
            match stmt {
                Statement::Expression { expr, .. } => {
                    self.find_references_in_expression(expr, symbols)
                }
                Statement::Return { expr, .. } => self.find_references_in_expression(expr, symbols),
                Statement::VariableDeclaration {
                    initializer: Some(expr),
                    ..
                } => {
                    self.find_references_in_expression(expr, symbols);
                }
                Statement::VariableAssignment { value, .. } => {
                    self.find_references_in_expression(value, symbols);
                }
                Statement::PointerAssignment { pointer, value, .. } => {
                    self.find_references_in_expression(pointer, symbols);
                    self.find_references_in_expression(value, symbols);
                }
                _ => {}
            }
        }
    }

    pub(super) fn find_references_in_expression(
        &self,
        expr: &Expression,
        symbols: &mut HashMap<String, SymbolInfo>,
    ) {
        match expr {
            Expression::Identifier(name) => {
                // Track reference to this identifier
                if let Some(_symbol) = symbols.get_mut(name) {
                    // Add reference location (would need position info)
                }
            }
            Expression::FunctionCall { name, args, .. } => {
                // Track function call reference
                if let Some(_symbol) = symbols.get_mut(name) {
                    // Add reference location
                }
                // Recurse into arguments
                for arg in args {
                    self.find_references_in_expression(arg, symbols);
                }
            }
            Expression::MethodCall {
                object,
                method: _,
                args,
                ..
            } => {
                // Track UFC method call - recurse into object and args
                self.find_references_in_expression(object, symbols);
                for arg in args {
                    self.find_references_in_expression(arg, symbols);
                }
            }
            Expression::BinaryOp { left, right, .. } => {
                self.find_references_in_expression(left, symbols);
                self.find_references_in_expression(right, symbols);
            }
            Expression::MemberAccess { object, .. } => {
                self.find_references_in_expression(object, symbols);
            }
            Expression::ArrayIndex { array, index } => {
                self.find_references_in_expression(array, symbols);
                self.find_references_in_expression(index, symbols);
            }
            Expression::Conditional { scrutinee, arms } => {
                self.find_references_in_expression(scrutinee, symbols);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.find_references_in_expression(guard, symbols);
                    }
                    self.find_references_in_expression(&arm.body, symbols);
                }
            }
            Expression::Closure { body, .. } => {
                // Recurse into closure body expression
                self.find_references_in_expression(body, symbols);
            }
            Expression::Block(stmts) => {
                self.find_references_in_statements(stmts, symbols);
            }
            _ => {}
        }
    }
}
