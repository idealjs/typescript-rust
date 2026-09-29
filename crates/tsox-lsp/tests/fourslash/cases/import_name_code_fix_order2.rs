use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn import_name_code_fix_order2() {
    let content = r#"// @Filename: /a.ts
export const _aB: number;
export const _Ab: number;
export const aB: number;
export const Ab: number;
// @Filename: /b.ts
[|import {
    _aB,
    _Ab,
    Ab,
} from "./a";
aB;|]
// @Filename: /c.ts
[|import {
    _aB,
    _Ab,
    Ab,
} from "./a";
aB;|]"#;
    let mut s = Session::new_for_test("importNameCodeFix_order2", content);
    fourslash::go_to_file(&mut s, "/b.ts");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_file(&mut s, "/c.ts");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
