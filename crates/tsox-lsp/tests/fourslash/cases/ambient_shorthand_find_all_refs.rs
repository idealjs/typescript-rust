use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn ambient_shorthand_find_all_refs() {
    let content = r#"// @Filename: declarations.d.ts
declare module "jquery";
// @Filename: user.ts
import {/*1*/x} from "jquery";
// @Filename: user2.ts
import {/*2*/x} from "jquery";"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2")
}
