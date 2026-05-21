use super::*;

impl TypeChecker {
    pub(super) fn collected_value_type_param_scope(&self, name: &str) -> Option<HashSet<String>> {
        self.functions
            .get(name)
            .or_else(|| self.methods.get(name))
            .map(|info| info.type_params.iter().cloned().collect())
    }

    pub(super) fn collected_type_type_param_scope(&self, name: &str) -> Option<HashSet<String>> {
        self.structs
            .get(name)
            .map(|info| info.type_params.iter().cloned().collect())
            .or_else(|| {
                self.enums
                    .get(name)
                    .map(|info| info.type_params.iter().cloned().collect())
            })
    }

    pub(super) fn collected_behavior_type_param_scope(
        &self,
        name: &str,
    ) -> Option<HashSet<String>> {
        self.behaviors
            .get(name)
            .map(|info| info.type_params.iter().cloned().collect())
    }

    pub(super) fn validate_collected_struct_type_references(
        &mut self,
        name: &str,
        scoped: &HashSet<String>,
        span: Span,
    ) {
        let Some(info) = self.structs.get(name).cloned() else {
            return;
        };
        for (_, ty) in &info.fields {
            self.validate_generic_type_ref_bounds(ty, scoped, span);
        }
    }

    pub(super) fn validate_collected_enum_type_references(
        &mut self,
        name: &str,
        scoped: &HashSet<String>,
        span: Span,
    ) {
        let Some(info) = self.enums.get(name).cloned() else {
            return;
        };
        for (_, payload) in &info.variants {
            if let Some(payload) = payload {
                self.validate_generic_type_ref_bounds(payload, scoped, span);
            }
        }
    }

    pub(super) fn validate_collected_behavior_type_references(
        &mut self,
        name: &str,
        scoped: &HashSet<String>,
        span: Span,
    ) {
        let Some(info) = self.behaviors.get(name).cloned() else {
            return;
        };
        for method in &info.methods {
            for param in &method.params {
                self.validate_generic_type_ref_bounds(&param.ty, scoped, span);
            }
            if let Some(return_type) = &method.return_type {
                self.validate_generic_type_ref_bounds(return_type, scoped, span);
            }
        }
    }

    pub(super) fn validate_collected_value_type_references(
        &mut self,
        name: &str,
        scoped: &HashSet<String>,
        span: Span,
    ) {
        let info = self
            .functions
            .get(name)
            .or_else(|| self.methods.get(name))
            .cloned();
        let Some(info) = info else {
            return;
        };

        for (_, ty) in &info.params {
            self.validate_generic_type_ref_bounds(ty, scoped, span);
        }
        self.validate_generic_type_ref_bounds(&info.return_type, scoped, span);
    }
}
