use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.GoToEOF"]
#[test]
fn type_check_after_resolve() {
    let content = r#"/*start*/class Point implements /*IPointRef*/IPoint {
    getDist() {
        ssss;
    }
}/*end*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("GoToEOF"); // f.GoToEOF(t)
    fourslash::unsupported("InsertLine"); // f.InsertLine(t, "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "IPointRef", "any", "")
    fourslash::unsupported("VerifyErrorExistsAfterMarker"); // f.VerifyErrorExistsAfterMarker(t, "IPointRef")
    fourslash::unsupported("GoToEOF"); // f.GoToEOF(t)
    fourslash::unsupported("InsertLine"); // f.InsertLine(t, "")
    fourslash::unsupported("VerifyErrorExistsAfterMarker"); // f.VerifyErrorExistsAfterMarker(t, "IPointRef")
}
