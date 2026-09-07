use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_export_as_namespace() {
    let content = r#"// @Filename: /node_modules/a/index.d.ts
export function /*0*/f(): void;
export as namespace A;
// @Filename: /b.ts
import { /*1*/f } from "a";
// @Filename: /c.ts
A./*2*/f();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "0", "1", "2")
}
