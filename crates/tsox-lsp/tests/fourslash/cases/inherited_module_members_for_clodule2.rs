use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNumberOfErrorsInCurrentFile"]
#[test]
fn inherited_module_members_for_clodule2() {
    let content = r#"// @strict: false
namespace M {
    export namespace A {
        var o;
    }
}
namespace M {
    export class A { a = 1;}
}
namespace M {
    export class A { /**/b }
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyQuickInfoExists"); // f.VerifyQuickInfoExists(t)
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 4)
}
