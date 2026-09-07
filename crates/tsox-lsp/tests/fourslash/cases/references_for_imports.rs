use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn references_for_imports() {
    let content = r#"declare module "jquery" {
    function $(s: string): any;
    export = $;
}
/*1*/import /*2*/$ = require("jquery");
/*3*/$("a");
/*4*/import /*5*/$ = require("jquery");"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5")
}
