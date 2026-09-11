use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_js_doc_tags6() {
    let content = r#"// @noEmit: true
// @allowJs: true
// @Filename: quickInfoJsDocTags6.js
class Foo {
    /**
     * comment
     * @author Me <me@domain.tld>
     * @see x (the parameter)
     * @param {number} x - x comment
     * @param {number} y - y comment
     * @returns The result
     */
    method(x, y) {
       return x + y;
    }
}

class Bar extends Foo {
    /** @inheritDoc */
    /**/method(x, y) {
        const res = super.method(x, y) + 100;
        return res;
    }
}"#;
    let mut s = Session::new_for_test("quickInfoJsDocTags6", content);
    // TODO: f.VerifyBaselineHover(t)
}
