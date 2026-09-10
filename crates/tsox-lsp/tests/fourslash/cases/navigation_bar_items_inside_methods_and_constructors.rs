use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_items_inside_methods_and_constructors() {
    let content = r#"class Class {
    constructor() {
        function LocalFunctionInConstructor() {}
        interface LocalInterfaceInConstrcutor {}
        enum LocalEnumInConstructor { LocalEnumMemberInConstructor }
    }

    method() {
        function LocalFunctionInMethod() {
            function LocalFunctionInLocalFunctionInMethod() {}
        }
        interface LocalInterfaceInMethod {}
        enum LocalEnumInMethod { LocalEnumMemberInMethod }
    }

    emptyMethod() { } // Non child functions method should not be duplicated
}"#;
    let mut s = Session::new_for_test("navigationBarItemsInsideMethodsAndConstructors", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
