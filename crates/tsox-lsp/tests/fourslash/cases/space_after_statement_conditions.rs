use tsox_lsp::fourslash::{self, Session};


#[test]
fn space_after_statement_conditions() {
    let content = r#"let i = 0;

if(i<0) ++i;
if(i<0) --i;

while(i<0) ++i;
while(i<0) --i;

do ++i;
while(i<0)
do --i;
while(i<0)

for(let prop in { foo: 1 }) ++i;
for(let prop in { foo: 1 }) --i;

for(let foo of [1, 2]) ++i;
for(let foo of [1, 2]) --i;

for(let j = 0; j < 10; j++) ++i;
for(let j = 0; j < 10; j++) --i;
"#;
    let mut s = Session::new_for_test("spaceAfterStatementConditions", content);
    // TODO: f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"let i = 0;

if (i < 0) ++i;
if (i < 0) --i;

while (i < 0) ++i;
while (i < 0) --i;

do ++i;
while (i < 0)
do --i;
while (i < 0)

for (let prop in { foo: 1 }) ++i;
for (let prop in { foo: 1 }) --i;

for (let foo of [1, 2]) ++i;
for (let foo of [1, 2]) --i;

for (let j = 0; j < 10; j++) ++i;
for (let j = 0; j < 10; j++) --i;
"#);
}
