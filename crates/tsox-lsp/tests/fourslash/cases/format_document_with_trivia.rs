use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_document_with_trivia() {
    let content = r#"  
// 1 below   
    
// 2 above   
    
let x;
  
// abc
  
let y;
  
// 3 above
   
while (true) {
    while (true) {
    }
      
    // 4 above   
}
  
// 5 above  
   
   "#;
    let mut s = Session::new(content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(
        &mut s,
        r#"
// 1 below   

// 2 above   

let x;

// abc

let y;

// 3 above

while (true) {
    while (true) {
    }

    // 4 above   
}

// 5 above  

"#,
    );
}
