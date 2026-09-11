use tsox_lsp::fourslash::{self, Session};


#[test]
fn references_for_merged_declarations7() {
    let content = r#"interface Foo { }
namespace Foo {
    export interface /*1*/Bar { }
    export module /*2*/Bar { export interface Baz { } }
    export function /*3*/Bar() { }
}

// module, value and type
import a2 = Foo./*4*/Bar;"#;
    let mut s = Session::new_for_test("referencesForMergedDeclarations7", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
