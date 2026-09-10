use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn completions_import_js_module_exports_assignment() {
    let content = r#"// @Filename: /home/src/workspaces/project/tsconfig.json
{ "compilerOptions": { "module": "commonjs", "allowJs": true, "lib": ["es5"] } }
// @Filename: /home/src/workspaces/project/third_party/marked/src/defaults.js
function getDefaults() {
  return {
    baseUrl: null,
  };
}

function changeDefaults(newDefaults) {
  module.exports.defaults = newDefaults;
}

module.exports = {
  defaults: getDefaults(),
  getDefaults,
  changeDefaults
};
// @Filename: /home/src/workspaces/project/index.ts
/**/"#;
    let mut s = Session::new_for_test("completionsImport_jsModuleExportsAssignment", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: opts666 := f.GetOptions()
    // TODO: opts666.FormatCodeSettings.NewLineCharacter = "\n"
    fourslash::unsupported("Configure"); // f.Configure(t, opts666)
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "d");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
