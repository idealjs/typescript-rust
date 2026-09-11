use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_references_across_multiple_projects_vs() {
    let content = r#"//@Filename: a.ts
/*1*/var /*2*/x: number;
//@Filename: b.ts
/// <reference path="a.ts" />
/*3*/x++;
//@Filename: c.ts
/// <reference path="a.ts" />
/*4*/x++;"#;
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyBaselineVSFindAllReferences(t, "1", "2", "3", "4")
}
