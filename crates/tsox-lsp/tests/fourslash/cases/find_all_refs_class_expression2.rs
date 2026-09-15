use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_class_expression2() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
exports./*0*/A = class {};
// @Filename: /b.js
import { /*1*/A } from "./a";
/*2*/A;"#;
    let _s = Session::new_for_test("findAllRefsClassExpression2", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "0", "1", "2")
}
