use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn ng_proxy4() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: tsconfig.json
{
    "compilerOptions": {
        "lib": ["es5"],
        "plugins": [
            { "name": "diagnostic-adder" }
        ]
    },
    "files": ["a.ts"]
}
// @Filename: a.ts
let x = [1, 2];
x/**/
"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 1)
}
