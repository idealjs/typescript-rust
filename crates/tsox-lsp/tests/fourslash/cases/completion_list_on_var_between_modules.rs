use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("completionListOnVarBetweenModules", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["C1", "C2"]);
}
