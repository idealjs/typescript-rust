use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_import_umd_default_no_crash1() {
    let content = r#"// @moduleResolution: bundler
// @allowJs: true
// @checkJs: true
// @Filename: /node_modules/dottie/package.json
{
  "name": "dottie",
  "main": "dottie.js"
}
// @Filename: /node_modules/dottie/dottie.js
(function (undefined) {
  var root = this;

  var Dottie = function () {};

  Dottie["default"] = function (object, path, value) {};

  if (typeof module !== "undefined" && module.exports) {
    exports = module.exports = Dottie;
  } else {
    root["Dottie"] = Dottie;
    root["Dot"] = Dottie;

    if (typeof define === "function") {
      define([], function () {
        return Dottie;
      });
    }
  }
})();
// @Filename: /src/index.js
/**/"#;
    let mut s = Session::new_for_test("completionsImport_umdDefaultNoCrash1", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
