use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_on_aliased_module() {
    let content = r#"namespace M {
    export namespace N {
        export function foo() { }
        function bar() { }
    }
}
import p = M.N;
p./**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
