use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyOutliningSpans"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyOutliningSpans"); // f.VerifyOutliningSpans(t)
}
