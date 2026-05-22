use super::*;

impl CEmitter {
    pub(super) fn emit_enum_match(
        &mut self,
        scrutinee: &TypedExpression,
        arms: &[TypedMatchArm],
        result_var: Option<&str>,
    ) {
        let scrut = self.emit_expr_inline(scrutinee);
        self.line(&format!("switch ({}.tag) {{", scrut));
        self.indent();
        for arm in arms {
            match &arm.pattern {
                TypedPattern::EnumVariant {
                    type_name,
                    variant,
                    bindings,
                } => {
                    let tag = format!("{}_{}", c_ident(type_name), c_ident(variant));
                    self.line(&format!("case {}: {{", tag));
                    self.indent();
                    for (binding_name, binding_ty) in bindings {
                        let ty = self.c_type(binding_ty);
                        self.line(&format!(
                            "{} {} = {}.data.{};",
                            ty,
                            c_ident(binding_name),
                            scrut,
                            c_ident(variant).to_lowercase()
                        ));
                    }
                    self.emit_block_body_with_result(&arm.body, result_var);
                    self.line("break;");
                    self.dedent();
                    self.line("}");
                }
                TypedPattern::Wildcard => {
                    self.line("default: {");
                    self.indent();
                    self.emit_block_body_with_result(&arm.body, result_var);
                    self.line("break;");
                    self.dedent();
                    self.line("}");
                }
                _ => {}
            }
        }
        self.dedent();
        self.line("}");
    }
}
