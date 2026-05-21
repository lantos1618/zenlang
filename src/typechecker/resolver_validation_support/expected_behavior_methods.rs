struct ExpectedBehaviorMethod {
    signature: MethodSignatureMetadata,
    metadata: BehaviorMethodTypeMetadata,
}

impl ExpectedBehaviorMethod {
    fn new(method: &ast::BehaviorMethod) -> Self {
        let signature = expected_value_signature_metadata(&method.params, &method.return_type, &[]);
        let parameter_type_names: Vec<_> = signature
            .params
            .iter()
            .map(|param| param.display.clone())
            .collect();
        let parameter_names: Vec<_> = signature
            .params
            .iter()
            .map(|param| param.name.clone())
            .collect();
        let parameter_types: Vec<_> = signature
            .params
            .into_iter()
            .map(|param| param.typed)
            .collect();

        Self {
            signature: (
                method.name.clone(),
                parameter_type_names,
                signature.return_type.display,
            ),
            metadata: BehaviorMethodTypeMetadata {
                name: method.name.clone(),
                parameter_names,
                parameter_types,
                return_type: signature.return_type.typed,
            },
        }
    }
}

struct ExpectedBehaviorMethodMetadata {
    signatures: Vec<MethodSignatureMetadata>,
    typed: Vec<BehaviorMethodTypeMetadata>,
}

impl ExpectedBehaviorMethodMetadata {
    fn from_methods(methods: &[ExpectedBehaviorMethod]) -> Self {
        Self {
            signatures: methods
                .iter()
                .map(|method| method.signature.clone())
                .collect(),
            typed: methods
                .iter()
                .map(|method| method.metadata.clone())
                .collect(),
        }
    }
}

#[derive(Clone, Copy)]
struct BehaviorMethodValidation {
    display_code: &'static str,
    typed_code: &'static str,
}

impl BehaviorMethodValidation {
    fn resolver_codes() -> Self {
        Self {
            display_code: "E0219",
            typed_code: "E0355",
        }
    }

    fn display_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!("resolver behavior symbol '{name}' has methods '{actual}', expected '{expected}'")
    }

    fn typed_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver behavior symbol '{name}' has typed methods '{actual}', expected '{expected}'"
        )
    }
}

fn expected_behavior_method_metadata(
    methods: &[ast::BehaviorMethod],
) -> Vec<ExpectedBehaviorMethod> {
    let mut expected = Vec::new();
    for method in methods {
        expected.push(ExpectedBehaviorMethod::new(method));
    }
    expected
}
