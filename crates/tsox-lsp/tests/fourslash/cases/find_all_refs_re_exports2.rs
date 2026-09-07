use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_re_exports2() {
    let content = r#"// @Filename: /a.ts
export function /*1*/foo(): void {}
// @Filename: /b.ts
import { foo as oof } from "./a";"#;
    let mut s = Session::new(content);
    fourslash::verify_no_errors(&mut s);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1")
}
