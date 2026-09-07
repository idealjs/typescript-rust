use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn references_for_ambients2() {
    let content = r#"// @Filename: /defA.ts
declare module "a" {
    /*1*/export type /*2*/T = number;
}
// @Filename: /defB.ts
declare module "b" {
    export import a = require("a");
    export const x: a./*3*/T;
}
// @Filename: /defC.ts
declare module "c" {
    import b = require("b");
    const x: b.a./*4*/T;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
