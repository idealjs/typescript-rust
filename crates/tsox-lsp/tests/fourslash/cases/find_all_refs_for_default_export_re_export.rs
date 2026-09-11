use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_for_default_export_re_export() {
    let content = r#"// @Filename: /export.ts
const /*0*/foo = 1;
export default /*1*/foo;
// @Filename: /re-export.ts
export { /*2*/default } from "./export";
// @Filename: /re-export-dep.ts
import /*3*/fooDefault from "./re-export";"#;
    let mut s = Session::new_for_test("findAllRefsForDefaultExport_reExport", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "0", "1", "2", "3")
}
