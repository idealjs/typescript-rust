use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_js_doc_no_crash1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @strict: true
// @checkJs: true
// @allowJs: true
// @filename: index.js
/**
 * @example
  <file name="glyphicons.css">
    @import url(//netdna.bootstrapcdn.com/bootstrap/3.0.0/css/bootstrap-glyphicons.css);
  </file>
  <example module="ngAnimate" deps="angular-animate.js" animations="true">
    <file name="animations.css">
      .animate-show.ng-hide-add.ng-hide-add-active,
      .animate-show.ng-hide-remove.ng-hide-remove-active {
        transition:all linear 0./**/5s;
      }
    </file>
  </example>
 */
var ngShowDirective = ['$animate', function($animate) {}];"#;
    let mut s = Session::new_for_test("completionsJSDocNoCrash1", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some(""), &["url"], &[]);
}
