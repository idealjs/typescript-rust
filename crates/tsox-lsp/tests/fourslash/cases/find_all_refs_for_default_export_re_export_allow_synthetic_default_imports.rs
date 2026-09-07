use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_for_default_export_re_export_allow_synthetic_default_imports() {
    let content = r#"// @allowSyntheticDefaultImports: true
// @module: commonjs
// @Filename: /export.ts
const /*0*/foo = 1;
export = /*1*/foo;
// @Filename: /re-export.ts
export { /*2*/default } from "./export";
// @Filename: /re-export-dep.ts
import /*3*/fooDefault from "./re-export";"#;
    let mut s = Session::new(content);
    fourslash::verify_no_errors(&mut s);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "0", "1", "2", "3")
}
