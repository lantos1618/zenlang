use std::str::FromStr;

use crate::ast::{Declaration, Expression, MatchArm, Program, Statement};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BuildTargetDslKind {
    Executable,
    Test,
    Library,
}

impl BuildTargetDslKind {
    const EXECUTABLE: &'static str = "Executable";
    const TEST: &'static str = "Test";
    const LIBRARY: &'static str = "Library";

    fn as_str(self) -> &'static str {
        match self {
            Self::Executable => Self::EXECUTABLE,
            Self::Test => Self::TEST,
            Self::Library => Self::LIBRARY,
        }
    }
}

impl fmt::Display for BuildTargetDslKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for BuildTargetDslKind {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            Self::EXECUTABLE => Ok(Self::Executable),
            Self::TEST => Ok(Self::Test),
            Self::LIBRARY => Ok(Self::Library),
            _ => Err(()),
        }
    }
}

impl BuildGraph {
    pub fn from_build_program(program: &Program) -> Result<Self, BuildGraphError> {
        let build_body = program
            .declarations
            .iter()
            .find_map(|decl| match decl {
                Declaration::Function { name, body, .. } if name == "build" => Some(body),
                _ => None,
            })
            .ok_or(BuildGraphError::MissingBuildFunction)?;

        let mut lowering = BuildProgramLowering::default();
        lowering.collect_expr(build_body);
        Self::from_input(lowering.into_input())
    }
}

#[derive(Default)]
struct BuildProgramLowering {
    targets: Vec<BuildTargetInput>,
    declared_host_effects: Vec<HostEffect>,
    used_host_effects: Vec<HostEffect>,
}

impl BuildProgramLowering {
    fn into_input(self) -> BuildGraphInput {
        BuildGraphInput {
            targets: self.targets,
            declared_host_effects: self.declared_host_effects,
            used_host_effects: self.used_host_effects,
        }
    }

    fn collect_expr(&mut self, expr: &Expression) {
        if let Some(effect) = env_read_effect(expr) {
            self.used_host_effects.push(effect);
        }
        if let Some(effect) = declared_env_read_effect(expr) {
            self.declared_host_effects.push(effect);
        }
        if let Some(target) = build_target_from_builder_add(expr) {
            self.targets.push(target);
        }

        match expr {
            Expression::BinaryOp { left, right, .. } => {
                self.collect_expr(left);
                self.collect_expr(right);
            }
            Expression::UnaryOp { operand, .. } => self.collect_expr(operand),
            Expression::FunctionCall { args, .. }
            | Expression::ArrayLiteral { elements: args, .. } => {
                for arg in args {
                    self.collect_expr(arg);
                }
            }
            Expression::MethodCall { receiver, args, .. } => {
                self.collect_expr(receiver);
                for arg in args {
                    self.collect_expr(arg);
                }
            }
            Expression::MemberAccess { object, .. } => self.collect_expr(object),
            Expression::IndexAccess { object, index, .. } => {
                self.collect_expr(object);
                self.collect_expr(index);
            }
            Expression::StructLiteral { fields, .. } => {
                for (_, field) in fields {
                    self.collect_expr(field);
                }
            }
            Expression::EnumVariant { payload, .. } => {
                if let Some(payload) = payload {
                    self.collect_expr(payload);
                }
            }
            Expression::Match {
                scrutinee, arms, ..
            } => {
                self.collect_expr(scrutinee);
                for MatchArm { guard, body, .. } in arms {
                    if let Some(guard) = guard {
                        self.collect_expr(guard);
                    }
                    self.collect_expr(body);
                }
            }
            Expression::WhileLoop {
                condition, body, ..
            }
            | Expression::If {
                condition,
                then_body: body,
                ..
            } => {
                self.collect_expr(condition);
                self.collect_expr(body);
                if let Expression::If {
                    else_body: Some(else_body),
                    ..
                } = expr
                {
                    self.collect_expr(else_body);
                }
            }
            Expression::Loop { body, .. } => self.collect_expr(body),
            Expression::Block {
                statements, expr, ..
            } => {
                for statement in statements {
                    self.collect_statement(statement);
                }
                if let Some(expr) = expr {
                    self.collect_expr(expr);
                }
            }
            Expression::Return { value, .. } => {
                if let Some(value) = value {
                    self.collect_expr(value);
                }
            }
            Expression::Closure { body, .. } => self.collect_expr(body),
            Expression::Cast { expr, .. } | Expression::Defer { expr, .. } => {
                self.collect_expr(expr)
            }
            Expression::StringInterpolation { parts, .. } => {
                for part in parts {
                    if let crate::ast::StringPart::Expr(expr) = part {
                        self.collect_expr(expr);
                    }
                }
            }
            Expression::Range { start, end, .. } => {
                self.collect_expr(start);
                self.collect_expr(end);
            }
            Expression::IntLiteral { .. }
            | Expression::FloatLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::BoolLiteral { .. }
            | Expression::CharLiteral { .. }
            | Expression::Identifier { .. }
            | Expression::LoopControl { .. }
            | Expression::Break { .. }
            | Expression::Continue { .. }
            | Expression::Error { .. } => {}
        }
    }

    fn collect_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::VarDecl { value, .. } | Statement::Expression { expr: value, .. } => {
                self.collect_expr(value);
            }
            Statement::Assignment { target, value, .. } => {
                self.collect_expr(target);
                self.collect_expr(value);
            }
            Statement::Block { stmts, .. } => {
                for stmt in stmts {
                    self.collect_statement(stmt);
                }
            }
        }
    }
}

fn build_target_from_builder_add(expr: &Expression) -> Option<BuildTargetInput> {
    let Expression::MethodCall {
        receiver,
        method,
        args,
        ..
    } = expr
    else {
        return None;
    };
    if method != "add"
        || !matches!(receiver.as_ref(), Expression::Identifier { name, .. } if name == "b")
    {
        return None;
    }
    let [arg] = args.as_slice() else {
        return None;
    };
    let Expression::StructLiteral { name, fields, .. } = arg else {
        return None;
    };
    match name.parse::<BuildTargetDslKind>().ok()? {
        BuildTargetDslKind::Executable => executable_target_from_fields(fields),
        BuildTargetDslKind::Test => test_target_from_fields(fields),
        BuildTargetDslKind::Library => library_target_from_fields(fields),
    }
}

fn executable_target_from_fields(fields: &[(String, Expression)]) -> Option<BuildTargetInput> {
    let target_name = string_field(fields, "name")?;
    let root_source_file =
        string_field(fields, "main").or_else(|| string_field(fields, "root_source_file"))?;
    let out_dir = string_field(fields, "out_dir")?;
    let dependencies = string_array_field(fields, "dependencies").unwrap_or_default();
    let features = string_array_field(fields, "features").unwrap_or_default();

    Some(BuildTargetInput {
        name: target_name,
        kind: BuildTargetKind::Executable {
            root_source_file: root_source_file.clone(),
            out_dir,
        },
        sources: vec![root_source_file],
        dependencies,
        features,
    })
}

fn test_target_from_fields(fields: &[(String, Expression)]) -> Option<BuildTargetInput> {
    let root_source_file =
        string_field(fields, "root").or_else(|| string_field(fields, "root_source_file"))?;
    let target_name =
        string_field(fields, "name").unwrap_or_else(|| target_name_from_root(&root_source_file));
    let dependencies = string_array_field(fields, "dependencies").unwrap_or_default();
    let features = string_array_field(fields, "features").unwrap_or_default();

    Some(BuildTargetInput {
        name: target_name,
        kind: BuildTargetKind::Test {
            root_source_file: root_source_file.clone(),
        },
        sources: vec![root_source_file],
        dependencies,
        features,
    })
}

fn library_target_from_fields(fields: &[(String, Expression)]) -> Option<BuildTargetInput> {
    let target_name = string_field(fields, "name")?;
    let exports = string_array_field(fields, "exports")?;
    let dependencies = string_array_field(fields, "dependencies").unwrap_or_default();
    let features = string_array_field(fields, "features").unwrap_or_default();
    if exports.is_empty() {
        return None;
    }

    Some(BuildTargetInput {
        name: target_name,
        kind: BuildTargetKind::Library {
            exports: exports.clone(),
        },
        sources: exports,
        dependencies,
        features,
    })
}

fn target_name_from_root(root: &str) -> String {
    std::path::Path::new(root)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("test")
        .to_string()
}

fn string_field(fields: &[(String, Expression)], field_name: &str) -> Option<String> {
    fields.iter().find_map(|(name, value)| {
        (name == field_name).then(|| match value {
            Expression::StringLiteral { value, .. } => Some(value.clone()),
            _ => None,
        })?
    })
}

fn string_array_field(fields: &[(String, Expression)], field_name: &str) -> Option<Vec<String>> {
    fields.iter().find_map(|(name, value)| {
        if name != field_name {
            return None;
        }
        let Expression::ArrayLiteral { elements, .. } = value else {
            return None;
        };
        elements
            .iter()
            .map(|element| match element {
                Expression::StringLiteral { value, .. } => Some(value.clone()),
                _ => None,
            })
            .collect()
    })
}

fn declared_env_read_effect(expr: &Expression) -> Option<HostEffect> {
    let Expression::Match {
        scrutinee, arms, ..
    } = expr
    else {
        return None;
    };
    let has_fallback = arms.iter().any(|arm| {
        matches!(
            &arm.pattern,
            crate::ast::Pattern::Enum { variant, .. } if variant == "Err"
        )
    });
    has_fallback.then(|| env_read_effect(scrutinee)).flatten()
}

fn env_read_effect(expr: &Expression) -> Option<HostEffect> {
    let Expression::MethodCall {
        receiver,
        method,
        args,
        ..
    } = expr
    else {
        return None;
    };
    if method != "env" || !is_builder_os(receiver) {
        return None;
    }
    let [Expression::StringLiteral { value, .. }] = args.as_slice() else {
        return None;
    };
    Some(HostEffect::ReadEnv(value.clone()))
}

fn is_builder_os(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::MemberAccess { object, field, .. }
            if field == "os"
                && matches!(object.as_ref(), Expression::Identifier { name, .. } if name == "b")
    )
}

#[cfg(test)]
mod tests {
    use super::BuildTargetDslKind;

    #[test]
    fn build_target_dsl_kind_owns_source_spelling() {
        assert_eq!(BuildTargetDslKind::Executable.as_str(), "Executable");
        assert_eq!(BuildTargetDslKind::Test.as_str(), "Test");
        assert_eq!(BuildTargetDslKind::Library.as_str(), "Library");
        assert_eq!("Executable".parse(), Ok(BuildTargetDslKind::Executable));
        assert_eq!("Test".parse(), Ok(BuildTargetDslKind::Test));
        assert_eq!("Library".parse(), Ok(BuildTargetDslKind::Library));
        assert!("Benchmark".parse::<BuildTargetDslKind>().is_err());
        assert_eq!(BuildTargetDslKind::Executable.to_string(), "Executable");
        assert_eq!(BuildTargetDslKind::Test.to_string(), "Test");
        assert_eq!(BuildTargetDslKind::Library.to_string(), "Library");
    }
}
