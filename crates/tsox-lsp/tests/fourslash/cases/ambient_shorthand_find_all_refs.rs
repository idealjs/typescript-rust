use tsox_lsp::fourslash::{self, Session};


#[test]
fn ambient_shorthand_find_all_refs() {
    let content = r#"// @Filename: declarations.d.ts
declare module "jquery";
// @Filename: user.ts
import {/*1*/x} from "jquery";
// @Filename: user2.ts
import {/*2*/x} from "jquery";"#;
    let mut s = Session::new_for_test("ambientShorthandFindAllRefs", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}
