use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_css_pattern_ambient_module() {
    let content = r#"// @esModuleInterop: true
// @Filename: index.css
/*2a*/html { font-size: 16px; }
// @Filename: types.ts
declare module /*2b*/"*.css" {
  const styles: any;
  export = styles;
}
// @Filename: index.ts
import styles from [|/*1*/"./index.css"|];"#;
    let _s = Session::new_for_test("goToDefinitionCSSPatternAmbientModule", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
