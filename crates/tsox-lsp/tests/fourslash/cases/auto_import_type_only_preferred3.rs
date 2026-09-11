use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn auto_import_type_only_preferred3() {
    let content = r#"// @module: esnext
// @moduleResolution: bundler
// @Filename: /a.ts
export class A {}
export class B {}
// @Filename: /b.ts
let x: A/*b*/;
// @Filename: /c.ts
import { A } from "./a";
new A();
let x: B/*c*/;
// @Filename: /d.ts
new A();
let x: B;
// @Filename: /ns.ts
export * as default from "./a";
// @Filename: /e.ts
let x: /*e*/ns.A;"#;
    let mut s = Session::new_for_test("autoImportTypeOnlyPreferred3", content);
    fourslash::go_to_marker(&mut s, "b");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_marker(&mut s, "c");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_file(&mut s, "/d.ts");
    // TODO: f.VerifyCodeFixAll(t, fourslash.VerifyCodeFixAllOptions{
    fourslash::go_to_marker(&mut s, "e");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
