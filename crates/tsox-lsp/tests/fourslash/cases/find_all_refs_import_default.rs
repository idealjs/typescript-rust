use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_import_default() {
    let content = r#"// @Filename: f.ts
export { foo as default };
function /*start*/foo(a: number, b: number) {
    return a + b;
}
// @Filename: b.ts
import bar from "./f";
bar(1, 2);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "start")
}
