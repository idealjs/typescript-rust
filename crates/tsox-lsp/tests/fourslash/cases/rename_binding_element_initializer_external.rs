use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRenameAtRangesWithText"]
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
    let mut s = Session::new_for_test("renameBindingElementInitializerExternal", content);
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "external")
}
