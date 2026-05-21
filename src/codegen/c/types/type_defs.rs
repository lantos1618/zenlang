use super::*;

impl CEmitter {
    pub(super) fn emit_type_def(&mut self, typedef: &TypedTypeDef) {
        let name = c_ident(&typedef.name);
        match &typedef.kind {
            TypeDefKind::Struct { fields } => {
                self.line(&format!("struct {} {{", name));
                self.indent();
                for (field_name, field_type) in fields {
                    self.line(&format!(
                        "{} {};",
                        self.c_type(field_type),
                        c_ident(field_name)
                    ));
                }
                self.dedent();
                self.line("};");
            }
            TypeDefKind::Enum { variants } => {
                self.line(&format!("enum {}_Tag {{", name));
                self.indent();
                for variant in variants {
                    self.line(&format!(
                        "{}_{} = {},",
                        name,
                        c_ident(&variant.name),
                        variant.tag
                    ));
                }
                self.dedent();
                self.line("};");
                self.blank();

                self.line(&format!("struct {} {{", name));
                self.indent();
                self.line(&format!("enum {}_Tag tag;", name));

                let has_payloads = variants.iter().any(|v| v.payload.is_some());
                if has_payloads {
                    self.line("union {");
                    self.indent();
                    for variant in variants {
                        if let Some(fields) = &variant.payload {
                            if fields.len() == 1 {
                                self.line(&format!(
                                    "{} {};",
                                    self.c_type(&fields[0].1),
                                    c_ident(&variant.name).to_lowercase()
                                ));
                            } else {
                                self.line("struct {");
                                self.indent();
                                for (fname, ftype) in fields {
                                    self.line(&format!(
                                        "{} {};",
                                        self.c_type(ftype),
                                        c_ident(fname)
                                    ));
                                }
                                self.dedent();
                                self.line(&format!(
                                    "}} {};",
                                    c_ident(&variant.name).to_lowercase()
                                ));
                            }
                        }
                    }
                    self.dedent();
                    self.line("} data;");
                }
                self.dedent();
                self.line("};");
            }
        }

        for method in &typedef.methods {
            self.blank();
            self.emit_function(method);
        }
    }
}
