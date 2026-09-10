use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixModuleSpecifiers"]
#[test]
fn import_name_code_fix_infer_ending_preference() {
    let content = r#"// @module: esnext
// @moduleResolution: bundler
// @Filename: /a.mts
export {};
// @Filename: /b.ts
export {};
// @Filename: /c.ts
export const c = 0;
// @Filename: /main.ts
import {} from "./a.mjs";
import {} from "./b";

c/**/;"#;
    let mut s = Session::new_for_test("importNameCodeFixInferEndingPreference", content);
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"./c"}, nil /*preferences*/)
}
