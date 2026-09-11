use tsox_lsp::fourslash::{self, Session};


#[test]
fn ng_proxy1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: tsconfig.json
{
    "compilerOptions": {
        "lib": ["es5"],
        "plugins": [
            { "name": "quickinfo-augmeneter", "message": "hello world" }
        ]
    },
    "files": ["a.ts"]
}
// @Filename: a.ts
let x = [1, 2];
x/**/
"#;
    let mut s = Session::new_for_test("ngProxy1", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyQuickInfoIs(t, "Proxied x: number[]hello world", "")
}
