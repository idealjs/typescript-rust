use tsox_lsp::fourslash::Session;


#[test]
fn correupted_try_expressions_dont_crash_getting_outline_spans() {
    let content = r#"try[| {
  var x = [
    {% try[||] %}|][|{% except %}|] 
  ]
} catch (e)[| {
  
}|]"#;
    let _s = Session::new_for_test("correuptedTryExpressionsDontCrashGettingOutlineSpans", content);
    // TODO: f.VerifyOutliningSpans(t)
}
