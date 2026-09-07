use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineInlayHints"]
#[test]
fn inlay_hints_multifile1() {
    let content = r#"// @Filename: /a.ts
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
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/b.ts");
    fourslash::unsupported("VerifyBaselineInlayHints"); // f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
