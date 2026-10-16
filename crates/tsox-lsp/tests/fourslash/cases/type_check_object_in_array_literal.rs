use tsox_lsp::fourslash::{self, Session};


#[test]
fn type_check_object_in_array_literal() {
    let content = r#"declare function create<T>(initialValues);
create([{}]);"#;
    let mut s = Session::new_for_test("typeCheckObjectInArrayLiteral", content);
    // TODO: f.GoToPosition(t, 0)
    fourslash::insert(&mut s, "");
}
