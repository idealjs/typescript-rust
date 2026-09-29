use tsox_lsp::fourslash::{self, Session};


#[test]
fn type_arg_completion() {
    let content = r#"class Base {
}
class Derived extends Base {
}
interface I1<T extends Base>{
}
var x1: I1<Deri/**/>;"#;
    let mut s = Session::new_for_test("typeArgCompletion", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some(""), &["Derived"], &[]);
}
