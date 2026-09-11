use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_occurrences_property_in_aliased_interface() {
    let content = r#"namespace m {
    export interface Foo {
        [|abc|]
    }
}

import Bar = m.Foo;

export interface I extends Bar {
    [|abc|]
}

class C implements Bar {
    [|abc|]
}

(new C()).[|abc|];"#;
    let mut s = Session::new_for_test("getOccurrencesPropertyInAliasedInterface", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
