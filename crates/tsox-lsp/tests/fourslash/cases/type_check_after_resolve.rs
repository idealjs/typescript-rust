use tsox_lsp::fourslash::{self, Session};


#[test]
fn type_check_after_resolve() {
    let content = r#"/*start*/class Point implements /*IPointRef*/IPoint {
    getDist() {
        ssss;
    }
}/*end*/"#;
    let mut s = Session::new_for_test("typeCheckAfterResolve", content);
    fourslash::go_to_eof(&mut s, );
    fourslash::insert_line(&mut s, "");
    fourslash::verify_quick_info_at(&mut s, "IPointRef", "any", "");
    // TODO: f.VerifyErrorExistsAfterMarker(t, "IPointRef")
    fourslash::go_to_eof(&mut s, );
    fourslash::insert_line(&mut s, "");
    // TODO: f.VerifyErrorExistsAfterMarker(t, "IPointRef")
}
