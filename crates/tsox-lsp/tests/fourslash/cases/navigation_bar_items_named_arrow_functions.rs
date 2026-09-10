use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_items_named_arrow_functions() {
    let content = r#"export const value = 2;
export const func = () => 2;
export const func2 = function() { };
export function exportedFunction() { }"#;
    let mut s = Session::new_for_test("navigationBarItemsNamedArrowFunctions", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
