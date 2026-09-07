use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_js_doc_tags5() {
    let content = r#"// @noEmit: true
// @allowJs: true
// @Filename: quickInfoJsDocTags5.js
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
    /**/method(x, y) {
        const res = super.method(x, y) + 100;
        return res;
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
