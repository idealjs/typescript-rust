use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_entry_for_class_members_static_when_base_type_is_not_resolved() {
    let content = r#"// @Filename: /a.ts
import React from 'react'
class Slider extends React.Component {
    static defau/**/ltProps = {
        onMouseDown: () => { },
        onMouseUp: () => { },
        unit: 'px',
    }
    handleChange = () => 10;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
