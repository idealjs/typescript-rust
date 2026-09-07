use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_import_specifier_property_name() {
    let content = r#"// @Filename: canada.ts
export interface /**/Ginger {}
// @Filename: dry.ts
import { Ginger as Ale } from './canada';"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
