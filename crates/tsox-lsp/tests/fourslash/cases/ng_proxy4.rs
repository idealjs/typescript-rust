use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn ng_proxy4() {
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
    let mut s = Session::new_for_test("ngProxy4", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
}
