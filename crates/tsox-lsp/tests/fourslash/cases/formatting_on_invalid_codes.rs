use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn formatting_on_invalid_codes() {
    let content = r#"/*1*/var a;var c          , b;var  $d
/*2*/var $e
/*3*/var f
/*4*/a++;b++;

/*5*/function        f     (     )        {
/*6*/    for (i = 0; i < 10; i++) {
/*7*/        k = abc + 123 ^ d;
/*8*/        a = XYZ[m  (a[b[c][d]])];
/*9*/        break;

/*10*/        switch ( variable){
/*11*/       case  1: abc += 425;
/*12*/break;
/*13*/case 404 : a [x--/2]%=3 ;
/*14*/                    break ;
/*15*/                case vari : v[--x ] *=++y*( m + n / k[z]);
/*16*/                for (a in b){
/*17*/             for (a = 0; a < 10; ++a) {
/*18*/              a++;--a;
/*19*/                   if (a == b) {
/*20*/                          a++;b--;
/*21*/                     }
/*22*/else
/*23*/if (a == c){
/*24*/++a;
/*25*/(--c)+=d;
/*26*/$c = $a + --$b;
/*27*/}
/*28*/if (a == b)
/*29*/if (a != b) {
/*30*/ if (a !== b)
/*31*/ if (a === b)
/*32*/ --a;
/*33*/ else
/*34*/  --a;
/*35*/  else {
/*36*/  a--;++b;
/*37*/a++
/*38*/                    }
/*39*/                    }
/*40*/                    }
/*41*/                    for (x in y) {
/*42*/m-=m;
/*43*/k=1+2+3+4;
/*44*/}
/*45*/}
/*46*/    break;

/*47*/    }
/*48*/    }
/*49*/    var a  ={b:function(){}};
/*50*/    return {a:1,b:2}
/*51*/}

/*52*/var z = 1;
/*53*/            for (i = 0; i < 10; i++)
/*54*/     for (j = 0; j < 10; j++)
/*55*/for (k = 0; k < 10; ++k) {
/*56*/z++;
/*57*/}

/*58*/for (k = 0; k < 10; k += 2) {
/*59*/z++;
/*60*/}

/*61*/    $(document).ready ();


/*62*/ function  pageLoad() {
/*63*/ $('#TextBox1' ) .     unbind   (  ) ;
/*64*/$('#TextBox1' ) . datepicker ( ) ;
/*65*/}

/*66*/        function pageLoad    (     )    {
/*67*/    var webclass=[
/*68*/                { 'student'     :/*69*/
/*70*/                { 'id': '1', 'name': 'Linda Jones', 'legacySkill': 'Access, VB 5.0' }
/*71*/        }   ,
/*72*/{    'student':/*73*/
/*74*/{'id':'2','name':'Adam Davidson','legacySkill':'Cobol,MainFrame'}
/*75*/}      ,
/*76*/    { 'student':/*77*/
/*78*/{   'id':'3','name':'Charles Boyer' ,'legacySkill':'HTML, XML'}
/*79*/}
/*80*/    ];

/*81*/$create(Sys.UI.DataView,{data:webclass},null,null,$get('SList'));

/*82*/}

/*83*/$( document ).ready(function(){
/*84*/alert('hello');
/*85*/    } ) ;"#;
    let mut s = Session::new_for_test("formattingOnInvalidCodes", content);
    // TODO: f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"var a; var c, b; var $d"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"var $e"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"var f"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"a++; b++;"#);
    fourslash::go_to_marker(&mut s, "5");
    fourslash::verify_current_line_content(&mut s, r#"function f() {"#);
    fourslash::go_to_marker(&mut s, "6");
    fourslash::verify_current_line_content(&mut s, r#"    for (i = 0; i < 10; i++) {"#);
    fourslash::go_to_marker(&mut s, "7");
    fourslash::verify_current_line_content(&mut s, r#"        k = abc + 123 ^ d;"#);
    fourslash::go_to_marker(&mut s, "8");
    fourslash::verify_current_line_content(&mut s, r#"        a = XYZ[m(a[b[c][d]])];"#);
    fourslash::go_to_marker(&mut s, "9");
    fourslash::verify_current_line_content(&mut s, r#"        break;"#);
    fourslash::go_to_marker(&mut s, "10");
    fourslash::verify_current_line_content(&mut s, r#"        switch (variable) {"#);
    fourslash::go_to_marker(&mut s, "11");
    fourslash::verify_current_line_content(&mut s, r#"            case 1: abc += 425;"#);
    fourslash::go_to_marker(&mut s, "12");
    fourslash::verify_current_line_content(&mut s, r#"                break;"#);
    fourslash::go_to_marker(&mut s, "13");
    fourslash::verify_current_line_content(&mut s, r#"            case 404: a[x-- / 2] %= 3;"#);
    fourslash::go_to_marker(&mut s, "14");
    fourslash::verify_current_line_content(&mut s, r#"                break;"#);
    fourslash::go_to_marker(&mut s, "15");
    fourslash::verify_current_line_content(&mut s, r#"            case vari: v[--x] *= ++y * (m + n / k[z]);"#);
    fourslash::go_to_marker(&mut s, "16");
    fourslash::verify_current_line_content(&mut s, r#"                for (a in b) {"#);
    fourslash::go_to_marker(&mut s, "17");
    fourslash::verify_current_line_content(&mut s, r#"                    for (a = 0; a < 10; ++a) {"#);
    fourslash::go_to_marker(&mut s, "18");
    fourslash::verify_current_line_content(&mut s, r#"                        a++; --a;"#);
    fourslash::go_to_marker(&mut s, "19");
    fourslash::verify_current_line_content(&mut s, r#"                        if (a == b) {"#);
    fourslash::go_to_marker(&mut s, "20");
    fourslash::verify_current_line_content(&mut s, r#"                            a++; b--;"#);
    fourslash::go_to_marker(&mut s, "21");
    fourslash::verify_current_line_content(&mut s, r#"                        }"#);
    fourslash::go_to_marker(&mut s, "22");
    fourslash::verify_current_line_content(&mut s, r#"                        else"#);
    fourslash::go_to_marker(&mut s, "23");
    fourslash::verify_current_line_content(&mut s, r#"                            if (a == c) {"#);
    fourslash::go_to_marker(&mut s, "24");
    fourslash::verify_current_line_content(&mut s, r#"                                ++a;"#);
    fourslash::go_to_marker(&mut s, "25");
    fourslash::verify_current_line_content(&mut s, r#"                                (--c) += d;"#);
    fourslash::go_to_marker(&mut s, "26");
    fourslash::verify_current_line_content(&mut s, r#"                                $c = $a + --$b;"#);
    fourslash::go_to_marker(&mut s, "27");
    fourslash::verify_current_line_content(&mut s, r#"                            }"#);
    fourslash::go_to_marker(&mut s, "28");
    fourslash::verify_current_line_content(&mut s, r#"                        if (a == b)"#);
    fourslash::go_to_marker(&mut s, "29");
    fourslash::verify_current_line_content(&mut s, r#"                            if (a != b) {"#);
    fourslash::go_to_marker(&mut s, "30");
    fourslash::verify_current_line_content(&mut s, r#"                                if (a !== b)"#);
    fourslash::go_to_marker(&mut s, "31");
    fourslash::verify_current_line_content(&mut s, r#"                                    if (a === b)"#);
    fourslash::go_to_marker(&mut s, "32");
    fourslash::verify_current_line_content(&mut s, r#"                                        --a;"#);
    fourslash::go_to_marker(&mut s, "33");
    fourslash::verify_current_line_content(&mut s, r#"                                    else"#);
    fourslash::go_to_marker(&mut s, "34");
    fourslash::verify_current_line_content(&mut s, r#"                                        --a;"#);
    fourslash::go_to_marker(&mut s, "35");
    fourslash::verify_current_line_content(&mut s, r#"                                else {"#);
    fourslash::go_to_marker(&mut s, "36");
    fourslash::verify_current_line_content(&mut s, r#"                                    a--; ++b;"#);
    fourslash::go_to_marker(&mut s, "37");
    fourslash::verify_current_line_content(&mut s, r#"                                    a++"#);
    fourslash::go_to_marker(&mut s, "38");
    fourslash::verify_current_line_content(&mut s, r#"                                }"#);
    fourslash::go_to_marker(&mut s, "39");
    fourslash::verify_current_line_content(&mut s, r#"                            }"#);
    fourslash::go_to_marker(&mut s, "40");
    fourslash::verify_current_line_content(&mut s, r#"                    }"#);
    fourslash::go_to_marker(&mut s, "41");
    fourslash::verify_current_line_content(&mut s, r#"                    for (x in y) {"#);
    fourslash::go_to_marker(&mut s, "42");
    fourslash::verify_current_line_content(&mut s, r#"                        m -= m;"#);
    fourslash::go_to_marker(&mut s, "43");
    fourslash::verify_current_line_content(&mut s, r#"                        k = 1 + 2 + 3 + 4;"#);
    fourslash::go_to_marker(&mut s, "44");
    fourslash::verify_current_line_content(&mut s, r#"                    }"#);
    fourslash::go_to_marker(&mut s, "45");
    fourslash::verify_current_line_content(&mut s, r#"                }"#);
    fourslash::go_to_marker(&mut s, "46");
    fourslash::verify_current_line_content(&mut s, r#"                break;"#);
    fourslash::go_to_marker(&mut s, "47");
    fourslash::verify_current_line_content(&mut s, r#"        }"#);
    fourslash::go_to_marker(&mut s, "48");
    fourslash::verify_current_line_content(&mut s, r#"    }"#);
    fourslash::go_to_marker(&mut s, "49");
    fourslash::verify_current_line_content(&mut s, r#"    var a = { b: function() { } };"#);
    fourslash::go_to_marker(&mut s, "50");
    fourslash::verify_current_line_content(&mut s, r#"    return { a: 1, b: 2 }"#);
    fourslash::go_to_marker(&mut s, "51");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
    fourslash::go_to_marker(&mut s, "52");
    fourslash::verify_current_line_content(&mut s, r#"var z = 1;"#);
    fourslash::go_to_marker(&mut s, "53");
    fourslash::verify_current_line_content(&mut s, r#"for (i = 0; i < 10; i++)"#);
    fourslash::go_to_marker(&mut s, "54");
    fourslash::verify_current_line_content(&mut s, r#"    for (j = 0; j < 10; j++)"#);
    fourslash::go_to_marker(&mut s, "55");
    fourslash::verify_current_line_content(&mut s, r#"        for (k = 0; k < 10; ++k) {"#);
    fourslash::go_to_marker(&mut s, "56");
    fourslash::verify_current_line_content(&mut s, r#"            z++;"#);
    fourslash::go_to_marker(&mut s, "57");
    fourslash::verify_current_line_content(&mut s, r#"        }"#);
    fourslash::go_to_marker(&mut s, "58");
    fourslash::verify_current_line_content(&mut s, r#"for (k = 0; k < 10; k += 2) {"#);
    fourslash::go_to_marker(&mut s, "59");
    fourslash::verify_current_line_content(&mut s, r#"    z++;"#);
    fourslash::go_to_marker(&mut s, "60");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
    fourslash::go_to_marker(&mut s, "61");
    fourslash::verify_current_line_content(&mut s, r#"$(document).ready();"#);
    fourslash::go_to_marker(&mut s, "62");
    fourslash::verify_current_line_content(&mut s, r#"function pageLoad() {"#);
    fourslash::go_to_marker(&mut s, "63");
    fourslash::verify_current_line_content(&mut s, r#"    $('#TextBox1').unbind();"#);
    fourslash::go_to_marker(&mut s, "64");
    fourslash::verify_current_line_content(&mut s, r#"    $('#TextBox1').datepicker();"#);
    fourslash::go_to_marker(&mut s, "65");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
    fourslash::go_to_marker(&mut s, "66");
    fourslash::verify_current_line_content(&mut s, r#"function pageLoad() {"#);
    fourslash::go_to_marker(&mut s, "67");
    fourslash::verify_current_line_content(&mut s, r#"    var webclass = ["#);
    fourslash::go_to_marker(&mut s, "68");
    fourslash::verify_current_line_content(&mut s, r#"        {"#);
    fourslash::go_to_marker(&mut s, "69");
    fourslash::verify_current_line_content(&mut s, r#"            'student':"#);
    fourslash::go_to_marker(&mut s, "70");
    fourslash::verify_current_line_content(&mut s, r#"                { 'id': '1', 'name': 'Linda Jones', 'legacySkill': 'Access, VB 5.0' }"#);
    fourslash::go_to_marker(&mut s, "71");
    fourslash::verify_current_line_content(&mut s, r#"        },"#);
    fourslash::go_to_marker(&mut s, "72");
    fourslash::verify_current_line_content(&mut s, r#"        {"#);
    fourslash::go_to_marker(&mut s, "73");
    fourslash::verify_current_line_content(&mut s, r#"            'student':"#);
    fourslash::go_to_marker(&mut s, "74");
    fourslash::verify_current_line_content(&mut s, r#"                { 'id': '2', 'name': 'Adam Davidson', 'legacySkill': 'Cobol,MainFrame' }"#);
    fourslash::go_to_marker(&mut s, "75");
    fourslash::verify_current_line_content(&mut s, r#"        },"#);
    fourslash::go_to_marker(&mut s, "76");
    fourslash::verify_current_line_content(&mut s, r#"        {"#);
    fourslash::go_to_marker(&mut s, "77");
    fourslash::verify_current_line_content(&mut s, r#"            'student':"#);
    fourslash::go_to_marker(&mut s, "78");
    fourslash::verify_current_line_content(&mut s, r#"                { 'id': '3', 'name': 'Charles Boyer', 'legacySkill': 'HTML, XML' }"#);
    fourslash::go_to_marker(&mut s, "79");
    fourslash::verify_current_line_content(&mut s, r#"        }"#);
    fourslash::go_to_marker(&mut s, "80");
    fourslash::verify_current_line_content(&mut s, r#"    ];"#);
    fourslash::go_to_marker(&mut s, "81");
    fourslash::verify_current_line_content(&mut s, r#"    $create(Sys.UI.DataView, { data: webclass }, null, null, $get('SList'));"#);
    fourslash::go_to_marker(&mut s, "82");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
    fourslash::go_to_marker(&mut s, "83");
    fourslash::verify_current_line_content(&mut s, r#"$(document).ready(function() {"#);
    fourslash::go_to_marker(&mut s, "84");
    fourslash::verify_current_line_content(&mut s, r#"    alert('hello');"#);
    fourslash::go_to_marker(&mut s, "85");
    fourslash::verify_current_line_content(&mut s, r#"});"#);
}
