use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn js_doc_generics2() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
/**
 * @param {T[]} arr
 * @param {(function(T):T)} valuator
 * @template T
 */
function SortFilter(arr,valuator)
{
    return arr;
}
var a/*1*/ = SortFilter([0, 1, 2], q/*2*/ => q);
var b/*3*/ = SortFilter([0, 1, 2], undefined);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "var a: number[]", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(parameter) q: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "var b: number[]", "")
}
