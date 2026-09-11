use tsox_lsp::fourslash::{self, Session};


#[test]
fn insert_argument_before_overloaded_constructor() {
    let content = r#"alert(/**/100);

class OverloadedMonster {
    constructor();
    constructor(name) { }
}"#;
    let mut s = Session::new_for_test("insertArgumentBeforeOverloadedConstructor", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "'1', ");
}
