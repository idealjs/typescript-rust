use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("completionListOnAliasedModule", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["foo"]);
}
