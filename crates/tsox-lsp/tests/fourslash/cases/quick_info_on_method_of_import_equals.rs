use tsox_lsp::fourslash::{self, Session};

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
    fourslash::verify_quick_info_at(&mut s, "", "(method) C<number>.m(): void", "");
}
