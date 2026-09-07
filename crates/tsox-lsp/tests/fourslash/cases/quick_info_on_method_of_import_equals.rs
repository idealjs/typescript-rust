use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_on_method_of_import_equals() {
    let content = r#"// @Filename: /a.d.ts
declare class C<T> {
    m(): void;
}
export = C;
// @Filename: /b.ts
import C = require("./a");
declare var x: C<number>;
x./**/m;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "(method) C<number>.m(): void", "")
}
