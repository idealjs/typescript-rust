use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyQuickInfoExists"]
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
    let mut s = Session::new_for_test("inheritedModuleMembersForClodule2", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyQuickInfoExists"); // f.VerifyQuickInfoExists(t)
    fourslash::verify_number_of_errors_in_current_file(&mut s, 4);
}
