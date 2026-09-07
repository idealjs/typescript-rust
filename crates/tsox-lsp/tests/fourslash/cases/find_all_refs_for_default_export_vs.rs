use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineVSFindAllReferences"]
#[test]
fn find_all_refs_for_default_export_vs() {
    let content = r#"// @Filename: a.ts
export default function /*def*/f() {}
// @Filename: b.ts
import /*deg*/g from "./a";
[|/*ref*/g|]();
// @Filename: c.ts
import { f } from "./a";"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyBaselineVSFindAllReferences"); // f.VerifyBaselineVSFindAllReferences(t, "def", "deg")
}
