use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_type_parameter_of_type_alias3() {
    let content = r#"type constructorType<T1, T2> = new <T/*1*/, /*2*/"#;
    let mut s = Session::new_for_test("completionListInTypeParameterOfTypeAlias3", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), nil)
}
