use super::*;

pub(super) struct GenericBehaviorActualMethod<'a> {
    pub(super) params: &'a [Param],
    pub(super) return_type: &'a Option<AstType>,
    pub(super) span: Span,
}

pub(super) fn generic_behavior_actual_method<'a>(
    methods: &'a [Declaration],
    required_name: &str,
) -> Option<GenericBehaviorActualMethod<'a>> {
    methods.iter().find_map(|method| match method {
        Declaration::Function {
            name,
            params,
            return_type,
            span,
            ..
        } if name == required_name => Some(GenericBehaviorActualMethod {
            params: params.as_slice(),
            return_type,
            span: *span,
        }),
        _ => None,
    })
}
