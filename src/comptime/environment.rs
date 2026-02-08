// Compile-time variable environment with lexical scoping

use crate::error::{CompileError, Result};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::ComptimeValue;

#[derive(Debug, Clone)]
pub struct Environment {
    pub(crate) variables: Rc<RefCell<HashMap<String, (ComptimeValue, bool)>>>,
    parent: Option<Box<Environment>>,
}

impl Default for Environment {
    fn default() -> Self {
        Environment {
            variables: Rc::new(RefCell::new(HashMap::new())),
            parent: None,
        }
    }
}

impl Environment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_parent(parent: Environment) -> Self {
        Environment {
            variables: Rc::new(RefCell::new(HashMap::new())),
            parent: Some(Box::new(parent)),
        }
    }

    pub fn define(&self, name: String, value: ComptimeValue, is_mutable: bool) {
        self.variables
            .borrow_mut()
            .insert(name, (value, is_mutable));
    }

    pub fn get(&self, name: &str) -> Option<ComptimeValue> {
        self.variables
            .borrow()
            .get(name)
            .map(|(value, _)| value.clone())
            .or_else(|| self.parent.as_ref()?.get(name))
    }

    pub fn set(
        &self,
        name: &str,
        value: ComptimeValue,
        span: Option<crate::error::Span>,
    ) -> Result<()> {
        let vars = self.variables.borrow();
        if let Some((_, is_mutable)) = vars.get(name) {
            if !is_mutable {
                return Err(CompileError::ComptimeError(
                    format!("Cannot assign to immutable variable '{}'", name),
                    span,
                ));
            }
            drop(vars);
            self.variables
                .borrow_mut()
                .insert(name.to_string(), (value, true));
            Ok(())
        } else {
            drop(vars);
            if let Some(parent) = &self.parent {
                parent.set(name, value, span)
            } else {
                Err(CompileError::ComptimeError(
                    format!("Undefined variable: {}", name),
                    span,
                ))
            }
        }
    }
}
