use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_binding_element_initializer_property() {
    let content = r#"function f([|{[|{| "contextRangeIndex": 0 |}required|], optional = [|required|]}: {[|[|{| "contextRangeIndex": 3 |}required|]: number,|] optional?: number}|]) {
    console.log("required", [|required|]);
    console.log("optional", optional);
}

f({[|[|{| "contextRangeIndex": 6 |}required|]: 10|]});"#;
    let mut s = Session::new_for_test("renameBindingElementInitializerProperty", content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[2], f.Ranges()[5], f.Ranges
}
