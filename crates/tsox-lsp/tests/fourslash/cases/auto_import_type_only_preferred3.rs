use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn auto_import_type_only_preferred3() {
    // TODO: t.Skip("Known failing fourslash test")
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
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_marker(&mut s, "c");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_file(&mut s, "/d.ts");
    fourslash::unsupported("VerifyCodeFixAll"); // f.VerifyCodeFixAll(t, fourslash.VerifyCodeFixAllOptions{
    fourslash::go_to_marker(&mut s, "e");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
