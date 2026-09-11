use tsox_lsp::fourslash::{self, Session};


#[test]
fn add_function_in_duplicated_constructor_class_body() {
    let content = r#"class Foo {
    constructor() { }
    constructor() { }
    /**/
}"#;
    let mut s = Session::new_for_test("addFunctionInDuplicatedConstructorClassBody", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "fn() { }");
    fourslash::verify_number_of_errors_in_current_file(&mut s, 2);
}
