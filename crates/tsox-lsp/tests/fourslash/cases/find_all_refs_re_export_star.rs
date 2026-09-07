use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_re_export_star() {
    let content = r#"// @Filename: /a.ts
export function /*0*/foo(): void {}
// @Filename: /b.ts
export * from "./a";
// @Filename: /c.ts
import { /*1*/foo } from "./b";"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "0", "1")
}
