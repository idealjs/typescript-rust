use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_display_parts_external_module_alias() {
    let content = r#"// @Filename: quickInfoDisplayPartsExternalModuleAlias_file0.ts
export namespace m1 {
    export class c {
    }
}
// @Filename: quickInfoDisplayPartsExternalModuleAlias_file1.ts
import /*1*/a1 = require(/*mod1*/"./quickInfoDisplayPartsExternalModuleAlias_file0");
new /*2*/a1.m1.c();
export import /*3*/a2 = require(/*mod2*/"./quickInfoDisplayPartsExternalModuleAlias_file0");
new /*4*/a2.m1.c();"#;
    let mut s = Session::new_for_test("quickInfoDisplayPartsExternalModuleAlias", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
