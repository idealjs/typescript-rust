use tsox_lsp::fourslash::{self, Session};


#[test]
fn generic_arity_enforcement_after_edit() {
    let content = r#"interface G<T, U> { }
/**/
var v4: G<G<any>, any>;"#;
    let mut s = Session::new_for_test("genericArityEnforcementAfterEdit", content);
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, " ");
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
}
