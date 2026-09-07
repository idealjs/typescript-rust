use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: }"]
#[test]
fn format_interface_with_missing_brace_and_later_template_string2() {
    let content = r#"
// @Filename: /FormCheck.tsx
interface FormCheckProps {

const FormCheck: DynamicRefForwardingComponent<'input', FormCheckProps> =
  React.forwardRef(
	    () => {
	      return <div className={` + "`" + `${bsPrefix}-reverse` + "`" + `} />;
    },
  );

FormCheck.displayName = 'FormCheck';
"#;
    let mut s = Session::new(content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::unsupported("VerifyCurrentFileContent"); // f.VerifyCurrentFileContent(t, `interface FormCheckProps {
    // TODO: }
}
