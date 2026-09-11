use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_references_of_json_module() {
    let content = r#"// @resolveJsonModule: true
// @module: commonjs
// @esModuleInterop: true
// @Filename: /foo.ts
/*1*/import /*2*/settings from "./settings.json";
/*3*/settings;
// @Filename: /settings.json
 {}"#;
    let mut s = Session::new_for_test("findAllReferencesOfJsonModule", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
