use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn type_arg_completion() {
    let content = r#"class Base {
}
class Derived extends Base {
}
interface I1<T extends Base>{
}
var x1: I1<Deri/**/>;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
