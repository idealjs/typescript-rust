use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn module_node_next_auto_import3() {
    let content = r#"// @Filename: /tsconfig.json
{ "compilerOptions": { "module": "nodenext" } }
// @Filename: /package.json
{ "type": "module" }
// @Filename: /mobx.d.mts
export declare function autorun(): void;
// @Filename: /index.ts
autorun/**/
// @Filename: /utils.ts
import "./mobx.mjs";"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
