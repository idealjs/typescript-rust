use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_labels() {
    let content = r#"/*label1Definition*/label1: while (true) {
    /*label2Definition*/label2: while (true) {
        break [|/*1*/label1|];
        continue [|/*2*/label2|];
        () => { break [|/*3*/label1|]; }
        continue /*4*/unknownLabel;
    }
}"#;
    let mut s = Session::new_for_test("goToDefinitionLabels", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1", "2", "3", "4")
}
