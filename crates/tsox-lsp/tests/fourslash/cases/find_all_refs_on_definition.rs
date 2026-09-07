use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_on_definition() {
    let content = r#"//@Filename: findAllRefsOnDefinition-import.ts
export class Test{

    constructor(){

    }

    /*1*/public /*2*/start(){
        return this;
    }

    public stop(){
        return this;
    }
}
//@Filename: findAllRefsOnDefinition.ts
import Second = require("./findAllRefsOnDefinition-import");

var second = new Second.Test()
second./*3*/start();
second.stop();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
