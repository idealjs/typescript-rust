use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyOutliningSpans"]
#[test]
fn correupted_try_expressions_dont_crash_getting_outline_spans() {
    let content = r#"try[| {
  var x = [
    {% try[||] %}|][|{% except %}|] 
  ]
} catch (e)[| {
  
}|]"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyOutliningSpans"); // f.VerifyOutliningSpans(t)
}
