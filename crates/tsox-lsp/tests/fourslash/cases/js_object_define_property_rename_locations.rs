use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRenameAtRangesWithText"]
#[test]
fn js_object_define_property_rename_locations() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @noEmit: true
// @Filename: index.js
var CircularList = (function () {
    var CircularList = function() {};
    Object.defineProperty(CircularList.prototype, "[|maxLength|]", { value: 0, writable: true });
    CircularList.prototype.push = function (value) {
        // ...
        this.[|maxLength|] + this.[|maxLength|]
    }
    return CircularList;
})()"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/)
}
