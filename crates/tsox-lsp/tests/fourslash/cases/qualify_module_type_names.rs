use tsox_lsp::fourslash::{self, Session};


#[test]
fn qualify_module_type_names() {
    let content = r#"namespace m { export class c { } };
function x(arg: m.c) { return arg; }
x(/**/"#;
    let mut s = Session::new_for_test("qualifyModuleTypeNames", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "x(arg: m.c): m.c"})
}
