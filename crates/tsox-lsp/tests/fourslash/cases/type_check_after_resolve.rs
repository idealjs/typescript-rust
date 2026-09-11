use tsox_lsp::fourslash::{self, Session};


#[test]
fn type_check_after_resolve() {
    let content = r#"/*start*/class Point implements /*IPointRef*/IPoint {
    getDist() {
        ssss;
    }
}/*end*/"#;
    let mut s = Session::new_for_test("typeCheckAfterResolve", content);
    // TODO: f.GoToEOF(t)
    // TODO: f.InsertLine(t, "")
    fourslash::verify_quick_info_at(&mut s, "IPointRef", "any", "");
    // TODO: f.VerifyErrorExistsAfterMarker(t, "IPointRef")
    // TODO: f.GoToEOF(t)
    // TODO: f.InsertLine(t, "")
    // TODO: f.VerifyErrorExistsAfterMarker(t, "IPointRef")
}
