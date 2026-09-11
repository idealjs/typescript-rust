use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_index_re_export_prefix() {
    let content = r#"// @module: nodenext
// @Filename: /package.json
{ "type": "module" }
// @Filename: /utils/sum/index.ts
export { sum } from "./sum.js";
// @Filename: /utils/sum/sum.ts
export const sum = 0;
// @Filename: /utils/sumAB.ts
sum/**/"#;
    let mut s = Session::new_for_test("autoImportIndexReExportPrefix", content);
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"./sum/index.js", "./sum/sum.js"}, &lsutil.UserPre
}
