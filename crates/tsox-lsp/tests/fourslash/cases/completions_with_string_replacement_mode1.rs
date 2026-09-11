use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_with_string_replacement_mode1() {
    let content = r#"interface TFunction {
    (_: 'login.title', __?: {}): string;
    (_: 'login.description', __?: {}): string;
    (_: 'login.sendEmailAgree', __?: {}): string;
    (_: 'login.termsOfUse', __?: {}): string;
    (_: 'login.privacyPolicy', __?: {}): string;
    (_: 'login.sendEmailButton', __?: {}): string;
    (_: 'login.emailInputPlaceholder', __?: {}): string;
    (_: 'login.errorWrongEmailTitle', __?: {}): string;
    (_: 'login.errorWrongEmailDescription', __?: {}): string;
    (_: 'login.errorGeneralEmailTitle', __?: {}): string;
    (_: 'login.errorGeneralEmailDescription', __?: {}): string;
    (_: 'login.loginErrorTitle', __?: {}): string;
    (_: 'login.loginErrorDescription', __?: {}): string;
    (_: 'login.openEmailAppErrorTitle', __?: {}): string;
    (_: 'login.openEmailAppErrorDescription', __?: {}): string;
    (_: 'login.openEmailAppErrorConfirm', __?: {}): string;
}
const f: TFunction = (() => {}) as any;
f('[|login./**/|]')"#;
    let mut s = Session::new_for_test("completionsWithStringReplacementMode1", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
