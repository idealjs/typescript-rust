use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.Insert"]
#[test]
fn signature_help_import_star_from_export_equals() {
    let content = r#"// @allowJs: true
// @Filename: /node_modules/@types/abs/index.d.ts
declare function abs(str: string): string;
export = abs;
// @Filename: /a.js
import * as abs from "abs";
abs.default/**/;"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("Insert"); // f.Insert(t, "(")
}
