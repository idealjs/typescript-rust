use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn es_module_interop_find_all_references2() {
    let content = r#"// @esModuleInterop: true
// @Filename: /a.d.ts
export as namespace abc;
/*1*/export const /*2*/x: number;
// @Filename: /b.ts
import a from "./a";
a./*3*/x;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
