use tsox_lsp::fourslash::Session;


#[test]
fn doc_comment_template_returns_tag1() {
    let content = r#"/*0*/
function f1() {}
/*1*/
function f2() {
    return 1;
}
/*2*/
const f3 = () => 1;
/*3*/
const f3 = () => {
    return 1;
}
class Foo {
    /*4*/
    m1() {}

    /*5*/
    m2() {
       return 1;
    }
}"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let _s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyJSDocCompletion(t, "0", 3, `/** */`, nil)
    // TODO: f.VerifyJSDocCompletion(t, "1", 7, `/**
    // TODO: f.VerifyJSDocCompletion(t, "2", 7, `/**
    // TODO: f.VerifyJSDocCompletion(t, "3", 7, `/**
    // TODO: f.VerifyJSDocCompletion(t, "4", 3, `/** */`, nil)
    // TODO: f.VerifyJSDocCompletion(t, "5", 11, `/**
}
