use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("formatDocumentWithTrivia", content);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"
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

"#);
}
