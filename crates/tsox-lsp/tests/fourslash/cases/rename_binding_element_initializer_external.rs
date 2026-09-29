use tsox_lsp::fourslash::Session;


#[test]
fn rename_binding_element_initializer_external() {
    let content = r#"// @lib: es5
[|const [|{| "contextRangeIndex": 0 |}external|] = true;|]

function f({
    lvl1 = [|external|],
    nested: { lvl2 = [|external|]},
    oldName: newName = [|external|]
}) {}

const {
    lvl1 = [|external|],
    nested: { lvl2 = [|external|]},
    oldName: newName = [|external|]
} = obj;"#;
    let _s = Session::new_for_test("renameBindingElementInitializerExternal", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "external")
}
