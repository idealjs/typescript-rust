use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_import_equals_json_file() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @resolveJsonModule: true
// @module: commonjs
// @Filename: /a.ts
import /*0*/j = require("/*1*/./j.json");
/*2*/j;
// @Filename: /b.js
const /*3*/j = require("/*4*/./j.json");
/*5*/j;
// @Filename: /j.json
/*6*/{ "x": 0 }"#;
    let mut s = Session::new(content);
    fourslash::verify_no_errors(&mut s);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "0", "2", "1", "4", "3", "5", "6")
}
