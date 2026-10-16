use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_type_parameter_of_type_alias2() {
    let content = r#"type Map1<K, /*0*/
type Map1<K, /*1*/V> = [];
type Map1<K,V> = /*2*/[];
type Map1<K1, V1> = </*3*/"#;
    let mut s = Session::new_for_test("completionListInTypeParameterOfTypeAlias2", content);
    // TODO: f.VerifyCompletions(t, []string{"0", "1"}, nil)
    fourslash::verify_completions_include_exclude_at(&mut s, Some("2"), &["K", "V"], &[]);
    fourslash::verify_completions_empty_at(&mut s, Some("3"));
}
