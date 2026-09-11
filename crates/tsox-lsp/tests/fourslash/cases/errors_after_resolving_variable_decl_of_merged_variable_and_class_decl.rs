use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn errors_after_resolving_variable_decl_of_merged_variable_and_class_decl() {
    let content = r#"namespace M {
    export class C {
        foo() { }
    }
    export namespace C {
        export var /*1*/C = M.C;
    }
}"#;
    let mut s = Session::new_for_test("errorsAfterResolvingVariableDeclOfMergedVariableAndClassDecl", content);
    fourslash::verify_no_errors(&mut s, );
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.Backspace(t, 1)
    fourslash::insert(&mut s, " ");
    // TODO: f.VerifyQuickInfoIs(t, "var M.C.C: typeof M.C", "")
    fourslash::verify_no_errors(&mut s, );
}
