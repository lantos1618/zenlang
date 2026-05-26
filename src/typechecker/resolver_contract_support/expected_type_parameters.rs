struct ExpectedTypeParameter {
    name: String,
    bound: Option<ExpectedTypeParameterBound>,
}

impl ExpectedTypeParameter {
    fn new(type_param: &ast::TypeParam) -> Self {
        Self {
            name: type_param.name.clone(),
            bound: ExpectedTypeParameterBound::new(type_param),
        }
    }
}

struct ExpectedTypeParameterBound {
    display: TypeParameterBoundMetadata,
    reference: TypeParameterBoundRefMetadata,
}

impl ExpectedTypeParameterBound {
    fn new(type_param: &ast::TypeParam) -> Option<Self> {
        let behavior = type_param.constraint.as_ref()?;
        let display = type_param_bound_display(type_param)?;
        Some(Self {
            display: (type_param.name.clone(), display),
            reference: TypeParameterBoundRefMetadata {
                type_parameter: type_param.name.clone(),
                behavior: behavior.clone(),
                type_args: type_param.constraint_type_args.clone(),
            },
        })
    }
}

struct ExpectedTypeParameterMetadata {
    count: usize,
    names: Vec<String>,
    bounds: Vec<TypeParameterBoundMetadata>,
    bound_refs: Vec<TypeParameterBoundRefMetadata>,
}

impl ExpectedTypeParameterMetadata {
    fn from_parameters(parameters: &[ExpectedTypeParameter]) -> Self {
        Self {
            count: parameters.len(),
            names: parameters.iter().map(|param| param.name.clone()).collect(),
            bounds: parameters
                .iter()
                .filter_map(|param| param.bound.as_ref().map(|bound| bound.display.clone()))
                .collect(),
            bound_refs: parameters
                .iter()
                .filter_map(|param| param.bound.as_ref().map(|bound| bound.reference.clone()))
                .collect(),
        }
    }
}

#[derive(Clone, Copy)]
struct TypeParameterValidation {
    count_code: DiagnosticCode,
    name_code: DiagnosticCode,
    bound_code: DiagnosticCode,
    bound_ref_code: DiagnosticCode,
}

impl TypeParameterValidation {
    fn type_like_resolver_codes() -> Self {
        Self {
            count_code: ResolverContractCode::E0213.into(),
            name_code: ResolverContractCode::E0346.into(),
            bound_code: ResolverContractCode::E0222.into(),
            bound_ref_code: ResolverContractCode::E0350.into(),
        }
    }

    fn value_resolver_codes() -> Self {
        Self {
            count_code: ResolverContractCode::E0220.into(),
            name_code: ResolverContractCode::E0347.into(),
            bound_code: ResolverContractCode::E0221.into(),
            bound_ref_code: ResolverContractCode::E0351.into(),
        }
    }

    fn count_validation(self) -> CountValidation {
        CountValidation {
            label: "type parameter count",
            code: self.count_code,
        }
    }

    fn name_message(self, symbol_kind: &str, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver {symbol_kind} symbol '{name}' has type parameter names '{actual}', expected '{expected}'"
        )
    }

    fn bound_message(self, symbol_kind: &str, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver {symbol_kind} symbol '{name}' has type parameter bounds '{actual}', expected '{expected}'"
        )
    }

    fn bound_ref_message(
        self,
        symbol_kind: &str,
        name: &str,
        actual: &str,
        expected: &str,
    ) -> String {
        format!(
            "resolver {symbol_kind} symbol '{name}' has type parameter bound refs '{actual}', expected '{expected}'"
        )
    }
}
