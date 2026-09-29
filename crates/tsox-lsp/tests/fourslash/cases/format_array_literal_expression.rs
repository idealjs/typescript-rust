use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_array_literal_expression() {
    let content = r#"export let Things = [{
    Hat: 'hat', /*1*/
    Glove: 'glove',
    Umbrella: 'umbrella'
},{/*2*/
        Salad: 'salad', /*3*/
        Burrito: 'burrito',
        Pie: 'pie'
    }];/*4*/

export let Things2 = [
{
    Hat: 'hat', /*5*/
    Glove: 'glove',
    Umbrella: 'umbrella'
}/*6*/,
    {
        Salad: 'salad', /*7*/
        Burrito: ['burrito', 'carne asada', 'tinga de res', 'tinga de pollo'], /*8*/
        Pie: 'pie'
    }];/*9*/"#;
    let mut s = Session::new_for_test("formatArrayLiteralExpression", content);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"    Hat: 'hat',"#);
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCurrentLineContent(t, `
}
