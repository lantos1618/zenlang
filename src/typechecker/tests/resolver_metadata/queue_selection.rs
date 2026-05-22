use super::*;

#[test]
fn resolver_behavior_ref_queue_selection_prefers_exact_then_front() {
    let refs = VecDeque::from(vec![
        BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: vec![AstType::I32],
        },
        BehaviorRefMetadata {
            name: "Debug".to_string(),
            type_args: vec![],
        },
    ]);

    assert_eq!(
        TypeChecker::resolver_behavior_ref_queue_index(&refs, "Debug", &[]),
        Some(1)
    );
    assert_eq!(
        TypeChecker::resolver_behavior_ref_queue_index(&refs, "Missing", &[]),
        Some(0)
    );
    assert_eq!(
        TypeChecker::resolver_behavior_ref_queue_index(&VecDeque::new(), "Missing", &[]),
        None
    );
}

#[test]
fn resolver_behavior_ref_queue_selection_prefers_matching_type_args() {
    let refs = VecDeque::from(vec![
        BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: vec![AstType::Str],
        },
        BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: vec![AstType::I32],
        },
    ]);

    assert_eq!(
        TypeChecker::resolver_behavior_ref_queue_index(&refs, "Json", &[AstType::I32]),
        Some(1)
    );
    assert_eq!(
        TypeChecker::resolver_behavior_ref_queue_index(&refs, "Json", &[AstType::Bool]),
        Some(0)
    );
}

#[test]
fn named_queue_selection_prefers_exact_then_front() {
    let items = VecDeque::from(["Json".to_string(), "Debug".to_string()]);

    assert_eq!(
        TypeChecker::named_queue_index(&items, "Debug", String::as_str),
        Some(1)
    );
    assert_eq!(
        TypeChecker::named_queue_index(&items, "Missing", String::as_str),
        Some(0)
    );
    assert_eq!(
        TypeChecker::named_queue_index(&VecDeque::<String>::new(), "Missing", String::as_str),
        None
    );
}

#[test]
fn named_queue_selection_can_preserve_front_for_future_match() {
    let items = VecDeque::from(["Json".to_string(), "Debug".to_string()]);

    assert_eq!(
        TypeChecker::named_queue_index_preserving_future_front(
            &items,
            "Debug",
            Vec::<&str>::new(),
            String::as_str,
        ),
        Some(1)
    );
    assert_eq!(
        TypeChecker::named_queue_index_preserving_future_front(
            &items,
            "Missing",
            ["Json"],
            String::as_str,
        ),
        None
    );
    assert_eq!(
        TypeChecker::named_queue_index_preserving_future_front(
            &items,
            "Missing",
            ["Other"],
            String::as_str,
        ),
        Some(0)
    );
}
