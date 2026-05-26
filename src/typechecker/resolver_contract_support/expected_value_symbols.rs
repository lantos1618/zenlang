struct ExpectedValueSignature {
    params: Vec<ExpectedParameter>,
    return_type: ExpectedReturnMetadata,
    type_params: Vec<ExpectedTypeParameter>,
}

impl ExpectedValueSignature {
    fn new(
        params: &[Param],
        return_type: &Option<AstType>,
        type_params: &[ast::TypeParam],
    ) -> Self {
        Self {
            params: expected_parameter_metadata(params),
            return_type: expected_return_metadata(return_type),
            type_params: expected_type_parameter_metadata(type_params),
        }
    }
}

struct ExpectedValueSymbol {
    signature: ExpectedValueSignature,
    is_public: bool,
}

impl ExpectedValueSymbol {
    fn new(
        params: &[Param],
        return_type: &Option<AstType>,
        type_params: &[ast::TypeParam],
        is_public: bool,
    ) -> Self {
        Self {
            signature: ExpectedValueSignature::new(params, return_type, type_params),
            is_public,
        }
    }
}

struct ExpectedParameter {
    name: String,
    typed: AstType,
    display: String,
}

impl ExpectedParameter {
    fn new(name: &str, ty: &AstType) -> Self {
        Self {
            name: name.to_string(),
            typed: ty.clone(),
            display: ty.display_name(),
        }
    }
}

struct ExpectedParameterMetadata {
    count: usize,
    names: Vec<String>,
    display_types: Vec<String>,
    typed_types: Vec<AstType>,
}

#[derive(Clone, Copy)]
struct ValueParameterValidation {
    name_code: DiagnosticCode,
    display_type_code: DiagnosticCode,
    typed_type_code: DiagnosticCode,
}

impl ValueParameterValidation {
    fn resolver_codes() -> Self {
        Self {
            name_code: ResolverContractCode::E0223.into(),
            display_type_code: ResolverContractCode::E0216.into(),
            typed_type_code: ResolverContractCode::E0356.into(),
        }
    }

    fn name_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver value symbol '{name}' has parameter names '{actual}', expected '{expected}'"
        )
    }

    fn display_type_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver value symbol '{name}' has parameter types '{actual}', expected '{expected}'"
        )
    }

    fn typed_type_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver value symbol '{name}' has typed parameter types '{actual}', expected '{expected}'"
        )
    }
}

impl ExpectedParameterMetadata {
    fn from_parameters(parameters: &[ExpectedParameter]) -> Self {
        Self {
            count: parameters.len(),
            names: parameters.iter().map(|param| param.name.clone()).collect(),
            display_types: parameters
                .iter()
                .map(|param| param.display.clone())
                .collect(),
            typed_types: parameters.iter().map(|param| param.typed.clone()).collect(),
        }
    }
}

struct ExpectedReturnMetadata {
    typed: AstType,
    display: String,
}

impl ExpectedReturnMetadata {
    fn new(return_type: &Option<AstType>) -> Self {
        let typed = return_type.clone().unwrap_or(AstType::Void);
        Self {
            display: typed.display_name(),
            typed,
        }
    }
}

#[derive(Clone, Copy)]
struct ReturnValidation {
    display_code: DiagnosticCode,
    typed_code: DiagnosticCode,
}

impl ReturnValidation {
    fn resolver_codes() -> Self {
        Self {
            display_code: ResolverContractCode::E0212.into(),
            typed_code: ResolverContractCode::E0357.into(),
        }
    }

    fn display_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!("resolver value symbol '{name}' has return type '{actual}', expected '{expected}'")
    }

    fn typed_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver value symbol '{name}' has typed return type '{actual}', expected '{expected}'"
        )
    }
}
