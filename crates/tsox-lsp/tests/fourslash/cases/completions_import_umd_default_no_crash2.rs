use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_import_umd_default_no_crash2() {
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
import Dottie from 'dottie';
/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
