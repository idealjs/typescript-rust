use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineSignatureHelp"]
#[test]
fn quick_info_js_doc_text_formatting1_vs() {
    let content = r#"/**
 * @param {number} var1 **Highlighted text**
 * @param {string} var2 Another **Highlighted text**
*/
function f1(var1, var2) { }

/**
 * @param {number} var1 *Regular text with an asterisk
 * @param {string} var2 Another *Regular text with an asterisk
*/
function f2(var1, var2) { }

/**
 * @param {number} var1 
 * *Regular text with an asterisk
 * @param {string} var2 
 * Another *Regular text with an asterisk
*/
function f3(var1, var2) { }

/**
 * @param {number} var1 
 * **Highlighted text**
 * @param {string} var2 
 * Another **Highlighted text**
*/
function f4(var1, var2) { }

/**
 * @param {number} var1 
   **Highlighted text**
 * @param {string} var2 
   Another **Highlighted text**
*/
function f5(var1, var2) { }

f1(/*1*/);
f2(/*2*/);
f3(/*3*/);
f4(/*4*/);
f5(/*5*/);"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyBaselineSignatureHelp"); // f.VerifyBaselineSignatureHelp(t)
}
