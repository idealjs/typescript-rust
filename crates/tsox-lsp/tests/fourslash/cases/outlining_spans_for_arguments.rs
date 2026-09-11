use tsox_lsp::fourslash::{self, Session};


#[test]
fn outlining_spans_for_arguments() {
    let content = r#"console.log(123, 456)l;
console.log(
);
console.log[|(
    123, 456
)|];
console.log[|(
    123,
    456
)|];
() =>[| console.log[|(
    123,
    456
)|]|];"#;
    let mut s = Session::new_for_test("outliningSpansForArguments", content);
    // TODO: f.VerifyOutliningSpans(t)
}
