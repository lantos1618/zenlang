// Compile-time variable environment with lexical scoping

use crate::error::{CompileError, Result};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::ComptimeValue;

#[derive(Debug, Clone)]
pub struct Environment {
    pub(crate) variables: Rc<RefCell<HashMap<String, ComptimeValue>>>,
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

    pub fn define(&self, name: String, value: ComptimeValue) {
        self.variables.borrow_mut().insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<ComptimeValue> {
        self.variables
            .borrow()
            .get(name)
            .cloned()
            .or_else(|| self.parent.as_ref()?.get(name))
    }

    pub fn set(
        &self,
        name: &str,
        value: ComptimeValue,
        span: Option<crate::error::Span>,
    ) -> Result<()> {
        if self.variables.borrow().contains_key(name) {
            self.variables.borrow_mut().insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &self.parent {
            parent.set(name, value, span)
        } else {
            Err(CompileError::ComptimeError(
                format!("Undefined variable: {}", name),
                span,
            ))
        }
    }
}
