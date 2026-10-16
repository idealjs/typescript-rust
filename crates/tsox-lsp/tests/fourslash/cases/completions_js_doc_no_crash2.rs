use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_js_doc_no_crash2() {
    let content = r#"// @lib: es5
// @strict: true
// @filename: index.ts
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
    let mut s = Session::new_for_test("completionsJSDocNoCrash2", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
