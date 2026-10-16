use tsox_lsp::fourslash::{self, Session};


#[test]
fn module_node_next_auto_import1() {
    let content = r#"// @Filename: /tsconfig.json
{ "compilerOptions": { "module": "nodenext" } }
// @Filename: /package.json
{ "type": "module" }
// @Filename: /mobx.d.ts
export declare function autorun(): void;
// @Filename: /index.ts
autorun/**/
// @Filename: /utils.ts
import "./mobx.js";"#;
    let mut s = Session::new_for_test("moduleNodeNextAutoImport1", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
