use tsox_lsp::fourslash::{self, Session};


#[test]
fn inlay_hints_interactive_multifile1() {
    let content = r#"// @lib: es5
// @Filename: /a.ts
export interface Foo { a: string }
// @Filename: /b.ts
async function foo () {
    return {} as any as import('./a').Foo
}
function bar () { return import('./a') }
async function main () {
    const a = await foo()
    const b = await bar()
}"#;
    let mut s = Session::new_for_test("inlayHintsInteractiveMultifile1", content);
    fourslash::go_to_file(&mut s, "/b.ts");
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
