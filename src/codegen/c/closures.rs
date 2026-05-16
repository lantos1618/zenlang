use super::*;

impl CEmitter {
    /// Scan the program for closures and generate env struct + function definitions.
    pub(super) fn collect_closure_defs(&mut self, program: &TypedProgram) {
        for func in &program.functions {
            self.scan_block_for_closures(&func.body);
        }
    }

    fn scan_block_for_closures(&mut self, block: &TypedBlock) {
        for stmt in &block.statements {
            match &stmt.kind {
                TypedStatementKind::VarDecl { value, .. } => {
                    self.scan_expr_for_closures(value);
                }
                TypedStatementKind::Expression(e) => {
                    self.scan_expr_for_closures(e);
                }
            }
        }
        if let Some(ref e) = block.expr {
            self.scan_expr_for_closures(e);
        }
    }

    fn scan_expr_for_closures(&mut self, expr: &TypedExpression) {
        match &expr.kind {
            TypedExprKind::Closure {
                fn_name,
                env_type,
                captures,
            } => {
                let mut def = String::new();
                let fn_ident = c_ident(fn_name);

                if !captures.is_empty() && !env_type.is_empty() {
                    let env_ident = c_ident(env_type);
                    def.push_str(&format!("typedef struct {} {{\n", env_ident));
                    for cap in captures {
                        let ty = self.c_type(&cap.ty);
                        def.push_str(&format!("    {} {};\n", ty, c_ident(&cap.name)));
                    }
                    def.push_str(&format!("}} {};\n\n", env_ident));
                }

                if let Type::Function { params, ret } = &expr.ty {
                    let ret_str = self.c_type(ret);
                    let mut param_strs = Vec::new();
                    if !captures.is_empty() && !env_type.is_empty() {
                        param_strs.push(format!("{}* __env", c_ident(env_type)));
                    }
                    for (i, p) in params.iter().enumerate() {
                        param_strs.push(format!("{} __arg{}", self.c_type(p), i));
                    }
                    let params_str = if param_strs.is_empty() {
                        "void".to_string()
                    } else {
                        param_strs.join(", ")
                    };

                    def.push_str(&format!(
                        "static {} {}({}) {{\n",
                        ret_str, fn_ident, params_str
                    ));
                    def.push_str("    /* closure body emitted by caller */\n");
                    def.push_str("}\n");
                }

                self.closure_defs.push(def);
            }
            TypedExprKind::Block(block) => {
                self.scan_block_for_closures(block);
            }
            TypedExprKind::Match {
                scrutinee, arms, ..
            } => {
                self.scan_expr_for_closures(scrutinee);
                for arm in arms {
                    self.scan_block_for_closures(&arm.body);
                }
            }
            TypedExprKind::FunctionCall { args, .. } => {
                for arg in args {
                    self.scan_expr_for_closures(arg);
                }
            }
            TypedExprKind::BinaryOp { left, right, .. } => {
                self.scan_expr_for_closures(left);
                self.scan_expr_for_closures(right);
            }
            TypedExprKind::Assign { target, value } => {
                self.scan_expr_for_closures(target);
                self.scan_expr_for_closures(value);
            }
            _ => {}
        }
    }
}
