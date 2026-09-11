use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_interface_with_missing_brace_and_later_template_string2() {
    let content = r#"
// @Filename: /FormCheck.tsx
interface FormCheckProps {

const FormCheck: DynamicRefForwardingComponent<'input', FormCheckProps> =
  React.forwardRef(
	    () => {
	      return <div className={`${bsPrefix}-reverse`} />;
    },
  );

FormCheck.displayName = 'FormCheck';
"#;
    let mut s = Session::new_for_test("formatInterfaceWithMissingBraceAndLaterTemplateString2", content);
    // TODO: f.FormatDocument(t, "")
    // TODO: f.VerifyCurrentFileContent(t, `interface FormCheckProps {
    // TODO: }
}
