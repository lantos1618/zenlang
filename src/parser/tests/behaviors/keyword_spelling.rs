use crate::ast::declarations::TypeDeclarationKeyword;

#[test]
fn type_declaration_keyword_owns_text_spelling() {
    assert_eq!(TypeDeclarationKeyword::Impl.as_str(), "impl");
    assert_eq!(TypeDeclarationKeyword::Implements.as_str(), "implements");
    assert_eq!(TypeDeclarationKeyword::Requires.as_str(), "requires");
    assert_eq!(TypeDeclarationKeyword::Extends.as_str(), "extends");
    assert_eq!(TypeDeclarationKeyword::Derive.as_str(), "derive");
    assert_eq!(
        "impl".parse::<TypeDeclarationKeyword>(),
        Ok(TypeDeclarationKeyword::Impl)
    );
    assert_eq!(
        "implements".parse::<TypeDeclarationKeyword>(),
        Ok(TypeDeclarationKeyword::Implements)
    );
    assert_eq!(
        "requires".parse::<TypeDeclarationKeyword>(),
        Ok(TypeDeclarationKeyword::Requires)
    );
    assert_eq!(
        "extends".parse::<TypeDeclarationKeyword>(),
        Ok(TypeDeclarationKeyword::Extends)
    );
    assert_eq!(
        "derive".parse::<TypeDeclarationKeyword>(),
        Ok(TypeDeclarationKeyword::Derive)
    );
    assert!("implement".parse::<TypeDeclarationKeyword>().is_err());
    assert_eq!(TypeDeclarationKeyword::Implements.to_string(), "implements");
}
