use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.GoToPosition"]
#[test]
fn type_check_object_in_array_literal() {
    let content = r#"declare function create<T>(initialValues);
create([{}]);"#;
    let mut s = Session::new_for_test("typeCheckObjectInArrayLiteral", content);
    fourslash::unsupported("GoToPosition"); // f.GoToPosition(t, 0)
    fourslash::insert(&mut s, "");
}
