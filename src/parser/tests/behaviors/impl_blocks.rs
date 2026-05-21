use super::super::*;

#[test]
fn parse_behavior_impl_block() {
    let prog = parse_ok(
        "Point.implements(Json) {\n    to_json = (value: Point) StaticString { \"{}\" }\n}",
    );
    match &prog.declarations[0] {
        Declaration::ImplBlock {
            type_name,
            behavior,
            methods,
            ..
        } => {
            assert_eq!(type_name, "Point");
            assert_eq!(behavior.as_deref(), Some("Json"));
            assert_eq!(methods.len(), 1);
            assert_eq!(methods[0].name(), Some("to_json"));
        }
        other => panic!("expected ImplBlock, got {:?}", other),
    }
}

#[test]
fn parse_impl_block() {
    let prog = parse_ok("Point.impl = {\n    get = (self: Point) i32 { self.x }\n}");
    match &prog.declarations[0] {
        Declaration::ImplBlock {
            type_name,
            behavior,
            methods,
            ..
        } => {
            assert_eq!(type_name, "Point");
            assert_eq!(behavior, &None);
            assert_eq!(methods.len(), 1);
            assert_eq!(methods[0].name(), Some("get"));
        }
        other => panic!("expected ImplBlock, got {:?}", other),
    }
}

#[test]
fn parse_generic_impl_block_hoists_receiver_type_params_to_methods() {
    let prog = parse_ok("Box<T>.impl = {\n    get = (self: Box<T>) T { self.value }\n}");
    match &prog.declarations[0] {
        Declaration::ImplBlock {
            type_name,
            behavior,
            methods,
            ..
        } => {
            assert_eq!(type_name, "Box");
            assert_eq!(behavior, &None);
            assert_eq!(methods.len(), 1);
            let Declaration::Function { type_params, .. } = &methods[0] else {
                panic!("expected impl method function, got {:?}", methods[0]);
            };
            assert_eq!(type_params.len(), 1);
            assert_eq!(type_params[0].name, "T");
        }
        other => panic!("expected ImplBlock, got {:?}", other),
    }
}
