use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_on_definition2() {
    let content = r#"//@Filename: findAllRefsOnDefinition2-import.ts
export module Test{

    /*1*/export interface /*2*/start { }

    export interface stop { }
}
//@Filename: findAllRefsOnDefinition2.ts
import Second = require("./findAllRefsOnDefinition2-import");

var start: Second.Test./*3*/start;
var stop: Second.Test.stop;"#;
    let mut s = Session::new_for_test("findAllRefsOnDefinition2", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
