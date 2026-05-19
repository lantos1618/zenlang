use super::*;

impl CEmitter {
    pub(super) fn emit_struct_literal(
        &mut self,
        type_name: &str,
        fields: &[(String, TypedExpression)],
    ) -> String {
        let name = c_ident(type_name);
        let field_strs: Vec<_> = fields
            .iter()
            .map(|(fname, fval)| {
                let value = self.emit_expr_inline(fval);
                format!(".{} = {}", c_ident(fname), value)
            })
            .collect();
        format!("({}){{ {} }}", name, field_strs.join(", "))
    }

    pub(super) fn emit_enum_variant_literal(
        &mut self,
        type_name: &str,
        variant: &str,
        payload: Option<&TypedExpression>,
    ) -> String {
        let name = c_ident(type_name);
        let variant_ident = c_ident(variant);
        match payload {
            None => {
                format!("({}){{ .tag = {}_{} }}", name, name, variant_ident)
            }
            Some(value) => {
                let payload_value = self.emit_expr_inline(value);
                format!(
                    "({}){{ .tag = {}_{}, .data.{} = {} }}",
                    name,
                    name,
                    variant_ident,
                    variant_ident.to_lowercase(),
                    payload_value
                )
            }
        }
    }

    pub(super) fn emit_array_literal(&mut self, elements: &[TypedExpression]) -> String {
        let elems: Vec<_> = elements
            .iter()
            .map(|element| self.emit_expr_inline(element))
            .collect();
        format!("{{ {} }}", elems.join(", "))
    }
}
