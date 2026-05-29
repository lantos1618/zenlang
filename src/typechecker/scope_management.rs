use super::*;

impl TypeChecker {
    pub(crate) fn with_scope<T>(
        &mut self,
        check: impl FnOnce(&mut Self) -> Result<T, Diagnostic>,
    ) -> Result<T, Diagnostic> {
        self.scopes.push(HashMap::new());
        let result = check(self);
        self.scopes.pop();
        result
    }

    /// Push a persistent global scope (not auto-popped) for module-level
    /// constant bindings that must outlive individual function-body scopes.
    pub(crate) fn enter_global_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub(crate) fn define_var(&mut self, name: &str, ty: Type) {
        self.define_var_with_mutability(name, ty, false);
    }

    pub(crate) fn define_var_with_mutability(&mut self, name: &str, ty: Type, mutable: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), VarInfo { ty, mutable });
        }
    }

    pub(crate) fn lookup_var_info(&self, name: &str) -> Option<&VarInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }
}
