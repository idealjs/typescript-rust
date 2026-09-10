use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn is_definition_overloads() {
    let content = r#"function /*1*/f(x: number): void;
function /*2*/f(x: string): void;
function /*3*/f(x: number | string) { }

f(1);
f("a");"#;
    let mut s = Session::new_for_test("isDefinitionOverloads", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
