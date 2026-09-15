use tsox_lsp::fourslash::Session;


#[test]
fn rename_object_binding_element_property_name01() {
    let content = r#"interface I {
    [|[|{| "contextRangeIndex": 0 |}property1|]: number;|]
    property2: string;
}

var foo: I;
[|var { [|{| "contextRangeIndex": 2 |}property1|]: prop1 } = foo;|]"#;
    let _s = Session::new_for_test("renameObjectBindingElementPropertyName01", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "property1")
}
