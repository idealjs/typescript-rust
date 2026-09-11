use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_class_expression0() {
    let content = r#"// @Filename: /a.ts
export = class /*0*/A {
    m() { /*1*/A; }
};
// @Filename: /b.ts
import /*2*/A = require("./a");
/*3*/A;"#;
    let mut s = Session::new_for_test("findAllRefsClassExpression0", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "0", "1", "2", "3")
}
