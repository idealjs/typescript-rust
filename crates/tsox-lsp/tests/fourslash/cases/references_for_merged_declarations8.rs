use tsox_lsp::fourslash::{self, Session};


#[test]
fn references_for_merged_declarations8() {
    let content = r#"interface Foo { }
namespace Foo {
    export interface Bar { }
    /*1*/export module /*2*/Bar { export interface Baz { } }
    export function Bar() { }
}

// module
import a3 = Foo./*3*/Bar.Baz;"#;
    let mut s = Session::new_for_test("referencesForMergedDeclarations8", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
