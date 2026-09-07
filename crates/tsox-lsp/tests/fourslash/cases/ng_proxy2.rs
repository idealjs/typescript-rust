use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn ng_proxy2() {
    let content = r#"// @Filename: tsconfig.json
{
    "compilerOptions": {
        "lib": ["es5"],
        "plugins": [
            { "name": "invalidmodulename" }
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
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "let x: number[]", "")
}
