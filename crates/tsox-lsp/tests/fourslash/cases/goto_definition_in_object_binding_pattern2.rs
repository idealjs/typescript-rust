use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn goto_definition_in_object_binding_pattern2() {
    let content = r#"var p0 = ({a/*1*/a}) => {console.log(aa)};
function f2({ [|a/*a1*/1|], [|b/*b1*/1|] }: { /*a1_dest*/a1: number, /*b1_dest*/b1: number } = { a1: 0, b1: 0 }) {}"#;
    let mut s = Session::new_for_test("gotoDefinitionInObjectBindingPattern2", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1", "a1", "b1")
}
