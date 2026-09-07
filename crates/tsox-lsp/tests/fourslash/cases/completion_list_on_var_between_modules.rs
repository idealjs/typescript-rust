use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_on_var_between_modules() {
    let content = r#"namespace M1 {
    export class C1 {
    }
    export class C2 {
    }
}
var x: M1./**/
namespace M2 {
    export class Test3 {
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
