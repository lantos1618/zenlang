// Expression evaluation for the comptime interpreter

use crate::ast::{self, Expression, Pattern};
use crate::error::{CompileError, Result};
use std::collections::HashMap;

use super::values::*;
use super::ComptimeInterpreter;

impl ComptimeInterpreter {
    /// Evaluate an expression to a compile-time value
    pub fn evaluate_expression(
        &mut self,
        expr: &Expression,
        span: Option<crate::error::Span>,
    ) -> Result<ComptimeValue> {
        match expr {
            Expression::Integer32(v) => Ok(ComptimeValue::I32(*v)),
            Expression::Integer64(v) => Ok(ComptimeValue::I64(*v)),
            Expression::Float32(v) => Ok(ComptimeValue::F32(*v)),
            Expression::Float64(v) => Ok(ComptimeValue::F64(*v)),
            Expression::Boolean(v) => Ok(ComptimeValue::Bool(*v)),
            Expression::String(v) => Ok(ComptimeValue::String(v.clone())),

            Expression::Identifier(name) => {
                if let Some(module) = self.modules.get(name.as_str()) {
                    return Ok(module.clone());
                }

                self.env.get(name).ok_or_else(|| {
                    CompileError::ComptimeError(
                        format!("Undefined identifier: {}", name),
                        span.clone(),
                    )
                })
            }

            Expression::BinaryOp { left, op, right } => {
                let left_val = self.evaluate_expression(left, span.clone())?;
                let right_val = self.evaluate_expression(right, span.clone())?;
                self.evaluate_binary_op(left_val, op, right_val, span)
            }

            Expression::FunctionCall {
                module, name, args, ..
            } => self.evaluate_function_call(module.as_deref(), name, args, span),

            Expression::ArrayLiteral(elements) => {
                let values: Result<Vec<_>> = elements
                    .iter()
                    .map(|e| self.evaluate_expression(e, span.clone()))
                    .collect();
                Ok(ComptimeValue::Array(values?))
            }

            Expression::MemberAccess { object, member } => {
                let obj_val = self.evaluate_expression(object, span.clone())?;
                self.evaluate_member_access(obj_val, member, span)
            }

            Expression::MethodCall {
                object,
                method,
                args,
                ..
            } => {
                let obj_val = self.evaluate_expression(object, span.clone())?;
                self.evaluate_method_call(obj_val, method, args, span)
            }

            Expression::StdReference => self.modules.get("@std").cloned().ok_or_else(|| {
                CompileError::ComptimeError("@std module not available".to_string(), span)
            }),

            Expression::Comptime(inner) => self.evaluate_expression(inner, span),

            // Block expression: { stmt; stmt; expr }
            Expression::Block(statements) => {
                self.with_scope(|interp| {
                    let mut result = ComptimeValue::Void;
                    for stmt in statements {
                        match interp.execute_statement(stmt) {
                            Ok(Some(val)) => result = val,
                            Ok(None) => {}
                            Err(ComptimeSignal::Error(e)) => return Err(e),
                            // Tunnel control flow through the CompileError boundary
                            // so enclosing loops can catch it
                            Err(ComptimeSignal::Flow(cf)) => return Err(flow_to_error(&cf)),
                        }
                    }
                    Ok(result)
                })
            }

            // Pattern matching: scrutinee ? | pattern { body } | pattern2 { body2 }
            Expression::QuestionMatch { scrutinee, arms } => {
                let scrutinee_val = self.evaluate_expression(scrutinee, span.clone())?;
                self.evaluate_question_match(scrutinee_val, arms, span)
            }

            // Array indexing: arr[i]
            Expression::ArrayIndex { array, index } => {
                let arr_val = self.evaluate_expression(array, span.clone())?;
                let idx_val = self.evaluate_expression(index, span.clone())?;
                if let ComptimeValue::Array(items) = &arr_val {
                    if let Some(idx) = Self::value_to_index(&idx_val) {
                        if idx < items.len() {
                            Ok(items[idx].clone())
                        } else {
                            Err(CompileError::ComptimeError(
                                format!("Index {} out of bounds (len: {})", idx, items.len()),
                                span,
                            ))
                        }
                    } else {
                        Err(CompileError::ComptimeError(
                            format!("Cannot index array with {:?}", idx_val.get_type()),
                            span,
                        ))
                    }
                } else {
                    Err(CompileError::ComptimeError(
                        format!("Cannot index {:?}", arr_val.get_type()),
                        span,
                    ))
                }
            }

            // String interpolation: "Hello ${name}!"
            Expression::StringInterpolation { parts } => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        ast::StringPart::Literal(s) => result.push_str(s),
                        ast::StringPart::Interpolation(e) => {
                            let val = self.evaluate_expression(e, span.clone())?;
                            result.push_str(&format!("{}", val));
                        }
                    }
                }
                Ok(ComptimeValue::String(result))
            }

            // Struct literal: MyStruct { field1: val1, field2: val2 }
            Expression::StructLiteral { name, fields } => {
                let mut field_values = HashMap::new();
                for (field_name, field_expr) in fields {
                    let val = self.evaluate_expression(field_expr, span.clone())?;
                    field_values.insert(field_name.clone(), val);
                }
                Ok(ComptimeValue::Struct {
                    name: name.clone(),
                    fields: field_values,
                })
            }

            Expression::Range {
                start,
                end,
                inclusive,
            } => {
                let start_val = self.evaluate_expression(start, span.clone())?;
                let end_val = self.evaluate_expression(end, span.clone())?;

                match (start_val, end_val) {
                    (ComptimeValue::I32(start_i), ComptimeValue::I32(end_i)) => {
                        let end_val = if *inclusive {
                            end_i.checked_add(1).ok_or_else(|| {
                                CompileError::ComptimeError(
                                    "Inclusive range end overflows i32".to_string(),
                                    span.clone(),
                                )
                            })?
                        } else {
                            end_i
                        };

                        const MAX_COMPTIME_RANGE: i32 = 100_000;
                        let range_size = end_val.saturating_sub(start_i);
                        if range_size > MAX_COMPTIME_RANGE {
                            return Err(CompileError::ComptimeError(
                                format!(
                                    "Compile-time range too large: {} elements (max {})",
                                    range_size, MAX_COMPTIME_RANGE
                                ),
                                span.clone(),
                            ));
                        }

                        let mut values = Vec::with_capacity(range_size.max(0) as usize);
                        for i in start_i..end_val {
                            values.push(ComptimeValue::I32(i));
                        }

                        Ok(ComptimeValue::Array(values))
                    }
                    _ => Err(CompileError::ComptimeError(
                        "Range expressions only support integer bounds".to_string(),
                        span,
                    )),
                }
            }

            _ => Err(CompileError::ComptimeError(
                format!("Expression type not supported in comptime: {:?}", expr),
                span,
            )),
        }
    }

    /// Evaluate binary operations
    pub(super) fn evaluate_binary_op(
        &self,
        left: ComptimeValue,
        op: &ast::BinaryOperator,
        right: ComptimeValue,
        span: Option<crate::error::Span>,
    ) -> Result<ComptimeValue> {
        use ast::BinaryOperator;

        match (left, right) {
            (ComptimeValue::I32(l), ComptimeValue::I32(r)) => match op {
                BinaryOperator::Add => Ok(ComptimeValue::I32(l + r)),
                BinaryOperator::Subtract => Ok(ComptimeValue::I32(l - r)),
                BinaryOperator::Multiply => Ok(ComptimeValue::I32(l * r)),
                BinaryOperator::Divide => {
                    if r == 0 {
                        Err(CompileError::ComptimeError(
                            "Division by zero".to_string(),
                            span,
                        ))
                    } else {
                        Ok(ComptimeValue::I32(l / r))
                    }
                }
                BinaryOperator::Equals => Ok(ComptimeValue::Bool(l == r)),
                BinaryOperator::NotEquals => Ok(ComptimeValue::Bool(l != r)),
                BinaryOperator::LessThan => Ok(ComptimeValue::Bool(l < r)),
                BinaryOperator::LessThanEquals => Ok(ComptimeValue::Bool(l <= r)),
                BinaryOperator::GreaterThan => Ok(ComptimeValue::Bool(l > r)),
                BinaryOperator::GreaterThanEquals => Ok(ComptimeValue::Bool(l >= r)),
                _ => Err(CompileError::ComptimeError(
                    format!("Unsupported operation {:?} for I32", op),
                    span,
                )),
            },

            (ComptimeValue::Bool(l), ComptimeValue::Bool(r)) => match op {
                BinaryOperator::And => Ok(ComptimeValue::Bool(l && r)),
                BinaryOperator::Or => Ok(ComptimeValue::Bool(l || r)),
                BinaryOperator::Equals => Ok(ComptimeValue::Bool(l == r)),
                BinaryOperator::NotEquals => Ok(ComptimeValue::Bool(l != r)),
                _ => Err(CompileError::ComptimeError(
                    format!("Unsupported operation {:?} for Bool", op),
                    span,
                )),
            },

            (ComptimeValue::String(l), ComptimeValue::String(r)) => match op {
                BinaryOperator::Add => Ok(ComptimeValue::String(format!("{}{}", l, r))),
                BinaryOperator::Equals => Ok(ComptimeValue::Bool(l == r)),
                BinaryOperator::NotEquals => Ok(ComptimeValue::Bool(l != r)),
                _ => Err(CompileError::ComptimeError(
                    format!("Unsupported operation {:?} for String", op),
                    span,
                )),
            },

            _ => Err(CompileError::ComptimeError(
                "Type mismatch in binary operation".to_string(),
                span,
            )),
        }
    }

    /// Evaluate pattern matching (QuestionMatch): scrutinee ? | pattern { body }
    pub(super) fn evaluate_question_match(
        &mut self,
        scrutinee: ComptimeValue,
        arms: &[ast::MatchArm],
        span: Option<crate::error::Span>,
    ) -> Result<ComptimeValue> {
        for arm in arms {
            if let Some(bindings) = self.match_pattern(&scrutinee, &arm.pattern)? {
                let define_bindings = |interp: &mut Self| {
                    for (name, val) in &bindings {
                        interp.env.define(name.clone(), val.clone(), false);
                    }
                };

                if let Some(guard) = &arm.guard {
                    let guard_passed = self.with_scope(|interp| {
                        define_bindings(interp);
                        interp.evaluate_expression(guard, span.clone())
                    })?;

                    match guard_passed {
                        ComptimeValue::Bool(true) => {}
                        ComptimeValue::Bool(false) => continue,
                        _ => {
                            return Err(CompileError::ComptimeError(
                                "Guard condition must be boolean".to_string(),
                                span,
                            ))
                        }
                    }
                }

                return self.with_scope(|interp| {
                    define_bindings(interp);
                    interp.evaluate_expression(&arm.body, span.clone())
                });
            }
        }

        Err(CompileError::ComptimeError(
            "Non-exhaustive pattern match: no arm matched".to_string(),
            span,
        ))
    }

    /// Try to match a comptime value against a pattern.
    /// Returns Some(bindings) if matched, None if not.
    pub(super) fn match_pattern(
        &mut self,
        value: &ComptimeValue,
        pattern: &Pattern,
    ) -> Result<Option<Vec<(String, ComptimeValue)>>> {
        match pattern {
            Pattern::Wildcard => Ok(Some(vec![])),

            Pattern::Identifier(name) => Ok(Some(vec![(name.clone(), value.clone())])),

            Pattern::Literal(expr) => {
                let pat_val = self.evaluate_expression(expr, None)?;
                if value == &pat_val {
                    Ok(Some(vec![]))
                } else {
                    Ok(None)
                }
            }

            Pattern::Type { type_name, binding } => {
                let matches = matches!(
                    (type_name.as_str(), value),
                    ("true", ComptimeValue::Bool(true))
                        | ("false", ComptimeValue::Bool(false))
                        | ("i32", ComptimeValue::I32(_))
                        | ("i64", ComptimeValue::I64(_))
                        | ("f32", ComptimeValue::F32(_))
                        | ("f64", ComptimeValue::F64(_))
                        | ("String", ComptimeValue::String(_))
                        | ("bool", ComptimeValue::Bool(_))
                );

                if matches {
                    let mut bindings = vec![];
                    if let Some(bind_name) = binding {
                        bindings.push((bind_name.clone(), value.clone()));
                    }
                    Ok(Some(bindings))
                } else {
                    Ok(None)
                }
            }

            Pattern::EnumLiteral { variant, payload } => match value {
                ComptimeValue::String(s) if s == variant => Ok(Some(vec![])),
                _ => {
                    if let ComptimeValue::Struct { name, fields } = value {
                        if name == variant
                            || fields
                                .get("variant")
                                .map(|v| {
                                    if let ComptimeValue::String(s) = v {
                                        s == variant
                                    } else {
                                        false
                                    }
                                })
                                .unwrap_or(false)
                        {
                            let mut bindings = vec![];
                            if let Some(payload_pat) = payload {
                                if let Some(inner) = fields.get("payload") {
                                    if let Some(b) = self.match_pattern(inner, payload_pat)? {
                                        bindings.extend(b);
                                    } else {
                                        return Ok(None);
                                    }
                                }
                            }
                            return Ok(Some(bindings));
                        }
                    }
                    Ok(None)
                }
            },

            Pattern::Or(patterns) => {
                for pat in patterns {
                    if let Some(bindings) = self.match_pattern(value, pat)? {
                        return Ok(Some(bindings));
                    }
                }
                Ok(None)
            }

            Pattern::Range {
                start,
                end,
                inclusive,
            } => {
                let start_val = self.evaluate_expression(start, None)?;
                let end_val = self.evaluate_expression(end, None)?;
                match (value, &start_val, &end_val) {
                    (ComptimeValue::I32(v), ComptimeValue::I32(s), ComptimeValue::I32(e)) => {
                        let in_range = if *inclusive {
                            v >= s && v <= e
                        } else {
                            v >= s && v < e
                        };
                        Ok(if in_range {
                            Some(vec![])
                        } else {
                            None
                        })
                    }
                    _ => Ok(None),
                }
            }

            Pattern::Guard { pattern, condition } => {
                if let Some(bindings) = self.match_pattern(value, pattern)? {
                    let guard_result = self.with_scope(|interp| {
                        for (name, val) in &bindings {
                            interp.env.define(name.clone(), val.clone(), false);
                        }
                        interp.evaluate_expression(condition, None)
                    })?;
                    match guard_result {
                        ComptimeValue::Bool(true) => Ok(Some(bindings)),
                        _ => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            }

            _ => Err(CompileError::ComptimeError(
                format!("Pattern type not yet supported in comptime: {:?}", pattern),
                None,
            )),
        }
    }
}
