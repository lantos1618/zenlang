use crate::ast::{Expression, Statement, Declaration, BinaryOperator};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ComptimeValue {
    Integer8(i8),
    Integer16(i16),
    Integer32(i32),
    Integer64(i64),
    Unsigned8(u8),
    Unsigned16(u16),
    Unsigned32(u32),
    Unsigned64(u64),
    Float32(f32),
    Float64(f64),
    Boolean(bool),
    String(String),
    Array(Vec<ComptimeValue>),
    Struct(HashMap<String, ComptimeValue>),
    Void,
}

impl ComptimeValue {
    pub fn to_expression(&self) -> Expression {
        match self {
            ComptimeValue::Integer8(v) => Expression::Integer8(*v),
            ComptimeValue::Integer16(v) => Expression::Integer16(*v),
            ComptimeValue::Integer32(v) => Expression::Integer32(*v),
            ComptimeValue::Integer64(v) => Expression::Integer64(*v),
            ComptimeValue::Unsigned8(v) => Expression::Unsigned8(*v),
            ComptimeValue::Unsigned16(v) => Expression::Unsigned16(*v),
            ComptimeValue::Unsigned32(v) => Expression::Unsigned32(*v),
            ComptimeValue::Unsigned64(v) => Expression::Unsigned64(*v),
            ComptimeValue::Float32(v) => Expression::Float32(*v),
            ComptimeValue::Float64(v) => Expression::Float64(*v),
            ComptimeValue::Boolean(v) => Expression::Boolean(*v),
            ComptimeValue::String(v) => Expression::String(v.clone()),
            ComptimeValue::Array(values) => Expression::ArrayLiteral(
                values.iter().map(|v| v.to_expression()).collect()
            ),
            ComptimeValue::Struct(_) => panic!("Cannot convert struct to expression"),
            ComptimeValue::Void => panic!("Cannot convert void to expression"),
        }
    }
}

pub struct ComptimeEvaluator {
    pub variables: HashMap<String, ComptimeValue>,
    pub functions: HashMap<String, Declaration>,
}

impl ComptimeEvaluator {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn evaluate_expression(&mut self, expr: &Expression) -> Result<ComptimeValue, String> {
        match expr {
            Expression::Integer8(v) => Ok(ComptimeValue::Integer8(*v)),
            Expression::Integer16(v) => Ok(ComptimeValue::Integer16(*v)),
            Expression::Integer32(v) => Ok(ComptimeValue::Integer32(*v)),
            Expression::Integer64(v) => Ok(ComptimeValue::Integer64(*v)),
            Expression::Unsigned8(v) => Ok(ComptimeValue::Unsigned8(*v)),
            Expression::Unsigned16(v) => Ok(ComptimeValue::Unsigned16(*v)),
            Expression::Unsigned32(v) => Ok(ComptimeValue::Unsigned32(*v)),
            Expression::Unsigned64(v) => Ok(ComptimeValue::Unsigned64(*v)),
            Expression::Float32(v) => Ok(ComptimeValue::Float32(*v)),
            Expression::Float64(v) => Ok(ComptimeValue::Float64(*v)),
            Expression::Boolean(v) => Ok(ComptimeValue::Boolean(*v)),
            Expression::String(v) => Ok(ComptimeValue::String(v.clone())),
            
            Expression::Identifier(name) => {
                self.variables.get(name)
                    .cloned()
                    .ok_or_else(|| format!("Undefined variable: {}", name))
            }
            
            Expression::BinaryOp { left, op, right } => {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;
                self.evaluate_binary_op(left_val, op.clone(), right_val)
            }
            
            Expression::ArrayLiteral(elements) => {
                let mut values = Vec::new();
                for elem in elements {
                    values.push(self.evaluate_expression(elem)?);
                }
                Ok(ComptimeValue::Array(values))
            }
            
            Expression::Comptime(inner) => {
                self.evaluate_expression(inner)
            }
            
            Expression::FunctionCall { name, args } => {
                self.evaluate_function_call(name, args)
            }
            
            Expression::StructLiteral { name: _, fields } => {
                let mut struct_values = HashMap::new();
                for (field_name, field_expr) in fields {
                    struct_values.insert(
                        field_name.clone(),
                        self.evaluate_expression(field_expr)?
                    );
                }
                Ok(ComptimeValue::Struct(struct_values))
            }
            
            Expression::StructField { struct_, field } => {
                let obj_val = self.evaluate_expression(struct_)?;
                match obj_val {
                    ComptimeValue::Struct(ref fields) => {
                        fields.get(field)
                            .cloned()
                            .ok_or_else(|| format!("Field {} not found", field))
                    }
                    _ => Err("Field access on non-struct".to_string())
                }
            }
            
            Expression::ArrayIndex { array, index } => {
                let array_val = self.evaluate_expression(array)?;
                let index_val = self.evaluate_expression(index)?;
                
                match (array_val, index_val) {
                    (ComptimeValue::Array(ref arr), ComptimeValue::Integer32(idx)) => {
                        let idx = idx as usize;
                        arr.get(idx)
                            .cloned()
                            .ok_or_else(|| "Index out of bounds".to_string())
                    }
                    (ComptimeValue::String(ref s), ComptimeValue::Integer32(idx)) => {
                        let idx = idx as usize;
                        s.chars()
                            .nth(idx)
                            .map(|c| ComptimeValue::String(c.to_string()))
                            .ok_or_else(|| "Index out of bounds".to_string())
                    }
                    _ => Err("Invalid index operation".to_string())
                }
            }
            
            Expression::Range { start, end, inclusive } => {
                let start_val = self.evaluate_expression(start)?;
                let end_val = self.evaluate_expression(end)?;
                
                match (start_val, end_val) {
                    (ComptimeValue::Integer32(s), ComptimeValue::Integer32(e)) => {
                        let values: Vec<ComptimeValue> = if *inclusive {
                            (s..=e).map(|i| ComptimeValue::Integer32(i)).collect()
                        } else {
                            (s..e).map(|i| ComptimeValue::Integer32(i)).collect()
                        };
                        Ok(ComptimeValue::Array(values))
                    }
                    _ => Err("Range bounds must be integers".to_string())
                }
            }
            
            _ => Err(format!("Cannot evaluate expression at compile time: {:?}", expr))
        }
    }
    
    fn evaluate_binary_op(&self, left: ComptimeValue, op: BinaryOperator, right: ComptimeValue) -> Result<ComptimeValue, String> {
        match (left, right, op) {
            (ComptimeValue::Integer32(l), ComptimeValue::Integer32(r), op) => {
                match op {
                    BinaryOperator::Add => Ok(ComptimeValue::Integer32(l + r)),
                    BinaryOperator::Subtract => Ok(ComptimeValue::Integer32(l - r)),
                    BinaryOperator::Multiply => Ok(ComptimeValue::Integer32(l * r)),
                    BinaryOperator::Divide => {
                        if r == 0 {
                            Err("Division by zero".to_string())
                        } else {
                            Ok(ComptimeValue::Integer32(l / r))
                        }
                    }
                    BinaryOperator::Modulo => {
                        if r == 0 {
                            Err("Modulo by zero".to_string())
                        } else {
                            Ok(ComptimeValue::Integer32(l % r))
                        }
                    }
                    BinaryOperator::Equals => Ok(ComptimeValue::Boolean(l == r)),
                    BinaryOperator::NotEquals => Ok(ComptimeValue::Boolean(l != r)),
                    BinaryOperator::LessThan => Ok(ComptimeValue::Boolean(l < r)),
                    BinaryOperator::GreaterThan => Ok(ComptimeValue::Boolean(l > r)),
                    BinaryOperator::LessThanEquals => Ok(ComptimeValue::Boolean(l <= r)),
                    BinaryOperator::GreaterThanEquals => Ok(ComptimeValue::Boolean(l >= r)),
                    _ => Err("Invalid operation for integers".to_string())
                }
            }
            
            (ComptimeValue::Float64(l), ComptimeValue::Float64(r), op) => {
                match op {
                    BinaryOperator::Add => Ok(ComptimeValue::Float64(l + r)),
                    BinaryOperator::Subtract => Ok(ComptimeValue::Float64(l - r)),
                    BinaryOperator::Multiply => Ok(ComptimeValue::Float64(l * r)),
                    BinaryOperator::Divide => Ok(ComptimeValue::Float64(l / r)),
                    BinaryOperator::Equals => Ok(ComptimeValue::Boolean((l - r).abs() < f64::EPSILON)),
                    BinaryOperator::NotEquals => Ok(ComptimeValue::Boolean((l - r).abs() >= f64::EPSILON)),
                    BinaryOperator::LessThan => Ok(ComptimeValue::Boolean(l < r)),
                    BinaryOperator::GreaterThan => Ok(ComptimeValue::Boolean(l > r)),
                    BinaryOperator::LessThanEquals => Ok(ComptimeValue::Boolean(l <= r)),
                    BinaryOperator::GreaterThanEquals => Ok(ComptimeValue::Boolean(l >= r)),
                    _ => Err("Invalid operation for floats".to_string())
                }
            }
            
            (ComptimeValue::Boolean(l), ComptimeValue::Boolean(r), op) => {
                match op {
                    BinaryOperator::And => Ok(ComptimeValue::Boolean(l && r)),
                    BinaryOperator::Or => Ok(ComptimeValue::Boolean(l || r)),
                    BinaryOperator::Equals => Ok(ComptimeValue::Boolean(l == r)),
                    BinaryOperator::NotEquals => Ok(ComptimeValue::Boolean(l != r)),
                    _ => Err("Invalid operation for booleans".to_string())
                }
            }
            
            (ComptimeValue::String(ref l), ComptimeValue::String(ref r), BinaryOperator::StringConcat) => {
                Ok(ComptimeValue::String(format!("{}{}", l, r)))
            }
            
            (ComptimeValue::String(ref l), ComptimeValue::String(ref r), BinaryOperator::Equals) => {
                Ok(ComptimeValue::Boolean(l == r))
            }
            
            _ => Err("Type mismatch in binary operation".to_string())
        }
    }
    
    fn evaluate_function_call(&mut self, name: &str, args: &[Expression]) -> Result<ComptimeValue, String> {
        match name {
            "len" => {
                if args.len() != 1 {
                    return Err("len() expects exactly one argument".to_string());
                }
                let arg = self.evaluate_expression(&args[0])?;
                match arg {
                    ComptimeValue::Array(ref arr) => Ok(ComptimeValue::Integer32(arr.len() as i32)),
                    ComptimeValue::String(ref s) => Ok(ComptimeValue::Integer32(s.len() as i32)),
                    _ => Err("len() can only be called on arrays or strings".to_string())
                }
            }
            _ => {
                if let Some(func_decl) = self.functions.get(name).cloned() {
                    self.evaluate_user_function(func_decl, args)
                } else {
                    Err(format!("Unknown function: {}", name))
                }
            }
        }
    }
    
    fn evaluate_user_function(&mut self, func_decl: Declaration, args: &[Expression]) -> Result<ComptimeValue, String> {
        if let Declaration::Function(func) = func_decl {
            if func.args.len() != args.len() {
                return Err(format!("Function {} expects {} arguments, got {}", 
                    func.name, func.args.len(), args.len()));
            }
            
            let mut saved_vars = HashMap::new();
            for ((param_name, _param_type), arg) in func.args.iter().zip(args.iter()) {
                let arg_val = self.evaluate_expression(arg)?;
                if let Some(old_val) = self.variables.insert(param_name.clone(), arg_val) {
                    saved_vars.insert(param_name.clone(), old_val);
                }
            }
            
            let mut result = ComptimeValue::Void;
            for stmt in &func.body {
                if let Some(val) = self.evaluate_statement(stmt)? {
                    result = val;
                    break;
                }
            }
            
            for (name, val) in saved_vars {
                self.variables.insert(name, val);
            }
            
            Ok(result)
        } else {
            Err("Not a function declaration".to_string())
        }
    }
    
    pub fn evaluate_statement(&mut self, stmt: &Statement) -> Result<Option<ComptimeValue>, String> {
        match stmt {
            Statement::VariableDeclaration { name, initializer, .. } => {
                if let Some(init) = initializer {
                    let value = self.evaluate_expression(init)?;
                    self.variables.insert(name.clone(), value);
                }
                Ok(None)
            }
            
            Statement::ComptimeBlock(statements) => {
                for stmt in statements {
                    if let Some(val) = self.evaluate_statement(stmt)? {
                        return Ok(Some(val));
                    }
                }
                Ok(None)
            }
            
            Statement::Return(expr) => {
                Ok(Some(self.evaluate_expression(expr)?))
            }
            
            Statement::Expression(expr) => {
                self.evaluate_expression(expr)?;
                Ok(None)
            }
            
            _ => Err(format!("Cannot evaluate statement at compile time: {:?}", stmt))
        }
    }
    
    pub fn evaluate_declaration(&mut self, decl: &Declaration) -> Result<(), String> {
        match decl {
            Declaration::ComptimeBlock(statements) => {
                for stmt in statements {
                    self.evaluate_statement(stmt)?;
                }
                Ok(())
            }
            
            Declaration::Function(func) => {
                self.functions.insert(func.name.clone(), decl.clone());
                Ok(())
            }
            
            _ => Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_evaluate_simple_arithmetic() {
        let mut evaluator = ComptimeEvaluator::new();
        
        let expr = Expression::BinaryOp {
            left: Box::new(Expression::Integer32(10)),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Integer32(20)),
        };
        
        let result = evaluator.evaluate_expression(&expr).unwrap();
        assert!(matches!(result, ComptimeValue::Integer32(30)));
    }
    
    #[test]
    fn test_evaluate_variable() {
        let mut evaluator = ComptimeEvaluator::new();
        
        evaluator.variables.insert("x".to_string(), ComptimeValue::Integer32(42));
        
        let expr = Expression::Identifier("x".to_string());
        let result = evaluator.evaluate_expression(&expr).unwrap();
        assert!(matches!(result, ComptimeValue::Integer32(42)));
    }
    
    #[test]
    fn test_evaluate_array() {
        let mut evaluator = ComptimeEvaluator::new();
        
        let expr = Expression::ArrayLiteral(vec![
            Expression::Integer32(1),
            Expression::Integer32(2),
            Expression::Integer32(3),
        ]);
        
        let result = evaluator.evaluate_expression(&expr).unwrap();
        match result {
            ComptimeValue::Array(values) => {
                assert_eq!(values.len(), 3);
                assert!(matches!(values[0], ComptimeValue::Integer32(1)));
                assert!(matches!(values[1], ComptimeValue::Integer32(2)));
                assert!(matches!(values[2], ComptimeValue::Integer32(3)));
            }
            _ => panic!("Expected array")
        }
    }
}