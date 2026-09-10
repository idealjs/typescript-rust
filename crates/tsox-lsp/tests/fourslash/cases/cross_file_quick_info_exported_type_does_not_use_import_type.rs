use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn cross_file_quick_info_exported_type_does_not_use_import_type() {
    let content = r#"// @Filename: b.ts
export interface B {}
export function foob(): {
    x: B,
    y: B
} {
    return null as any;
}
// @Filename: a.ts
import { foob } from "./b";
const thing/*1*/ = foob(/*2*/);"#;
    let mut s = Session::new_for_test("crossFileQuickInfoExportedTypeDoesNotUseImportType", content);
    fourslash::verify_quick_info_at(&mut s, "1", "const thing: {\n    x: B;\n    y: B;\n}", "");
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foob(): { x: B; y: B; }"})
}
