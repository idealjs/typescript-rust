use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_on_import_aliases() {
    let content = r#"//@Filename: a.ts
export class /*0*/Class {
}
//@Filename: b.ts
import { /*1*/Class } from "./a";

var c = new /*2*/Class();
//@Filename: c.ts
export { /*3*/Class } from "./a";"#;
    let mut s = Session::new_for_test("findAllRefsOnImportAliases", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "0", "1", "2")
}
