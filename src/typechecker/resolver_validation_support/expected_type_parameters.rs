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
    count_code: &'static str,
    name_code: &'static str,
    bound_code: &'static str,
    bound_ref_code: &'static str,
}

impl TypeParameterValidation {
    fn type_like_resolver_codes() -> Self {
        Self {
            count_code: "E0213",
            name_code: "E0346",
            bound_code: "E0222",
            bound_ref_code: "E0350",
        }
    }

    fn value_resolver_codes() -> Self {
        Self {
            count_code: "E0220",
            name_code: "E0347",
            bound_code: "E0221",
            bound_ref_code: "E0351",
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
