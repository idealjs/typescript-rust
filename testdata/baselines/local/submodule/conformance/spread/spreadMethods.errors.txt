spreadMethods.ts(22,35): error TS1005: '{' expected.
spreadMethods.ts(22,36): error TS1012: Unexpected token.
spreadMethods.ts(22,38): error TS1005: ')' expected.
spreadMethods.ts(22,53): error TS1005: ',' expected.
spreadMethods.ts(23,1): error TS1005: ',' expected.
spreadMethods.ts(23,1): error TS2304: Cannot find name 'let'.
spreadMethods.ts(23,5): error TS1005: ',' expected.
spreadMethods.ts(23,5): error TS2304: Cannot find name 'si'.
spreadMethods.ts(23,8): error TS1005: ',' expected.
spreadMethods.ts(23,10): error TS1005: ',' expected.
spreadMethods.ts(23,12): error TS1005: ',' expected.
spreadMethods.ts(25,1): error TS2304: Cannot find name 'si'.
spreadMethods.ts(26,1): error TS2304: Cannot find name 'si'.
spreadMethods.ts(27,1): error TS2304: Cannot find name 'si'.
spreadMethods.ts(32,32): error TS1005: '{' expected.
spreadMethods.ts(32,33): error TS1012: Unexpected token.
spreadMethods.ts(32,35): error TS1005: ')' expected.
spreadMethods.ts(32,50): error TS1005: ',' expected.
spreadMethods.ts(33,1): error TS1005: ',' expected.
spreadMethods.ts(33,1): error TS2304: Cannot find name 'let'.
spreadMethods.ts(33,5): error TS1005: ',' expected.
spreadMethods.ts(33,5): error TS2304: Cannot find name 'so'.
spreadMethods.ts(33,8): error TS1005: ',' expected.
spreadMethods.ts(33,10): error TS1005: ',' expected.
spreadMethods.ts(33,12): error TS1005: ',' expected.
spreadMethods.ts(35,1): error TS2304: Cannot find name 'so'.
spreadMethods.ts(36,1): error TS2304: Cannot find name 'so'.
spreadMethods.ts(37,1): error TS2304: Cannot find name 'so'.


==== spreadMethods.ts (28 errors) ====
    class K {
        p = 12;
        m() { }
        get g() { return 0; }
    }
    interface I {
        p: number;
        m(): void;
        readonly g: number;
    }
    
    let k = new K()
    let sk = { ...k };
    let ssk = { ...k, ...k };
    sk.p;
    sk.m(); // error
    sk.g; // error
    ssk.p;
    ssk.m(); // error
    ssk.g; // error
    
    let i: I = { p: 12, m() { }, get g() { return 0; } };
                                      ~
!!! error TS1005: '{' expected.
                                       ~
!!! error TS1012: Unexpected token.
                                         ~
!!! error TS1005: ')' expected.
                                                        ~
!!! error TS1005: ',' expected.
    let si = { ...i };
    ~~~
!!! error TS1005: ',' expected.
    ~~~
!!! error TS2304: Cannot find name 'let'.
        ~~
!!! error TS1005: ',' expected.
        ~~
!!! error TS2304: Cannot find name 'si'.
           ~
!!! error TS1005: ',' expected.
             ~
!!! error TS1005: ',' expected.
               ~~~
!!! error TS1005: ',' expected.
    let ssi = { ...i, ...i };
    si.p;
    ~~
!!! error TS2304: Cannot find name 'si'.
    si.m(); // ok
    ~~
!!! error TS2304: Cannot find name 'si'.
    si.g; // ok
    ~~
!!! error TS2304: Cannot find name 'si'.
    ssi.p;
    ssi.m(); // ok
    ssi.g; // ok
    
    let o = { p: 12, m() { }, get g() { return 0; } };
                                   ~
!!! error TS1005: '{' expected.
                                    ~
!!! error TS1012: Unexpected token.
                                      ~
!!! error TS1005: ')' expected.
                                                     ~
!!! error TS1005: ',' expected.
    let so = { ...o };
    ~~~
!!! error TS1005: ',' expected.
    ~~~
!!! error TS2304: Cannot find name 'let'.
        ~~
!!! error TS1005: ',' expected.
        ~~
!!! error TS2304: Cannot find name 'so'.
           ~
!!! error TS1005: ',' expected.
             ~
!!! error TS1005: ',' expected.
               ~~~
!!! error TS1005: ',' expected.
    let sso = { ...o, ...o };
    so.p;
    ~~
!!! error TS2304: Cannot find name 'so'.
    so.m(); // ok
    ~~
!!! error TS2304: Cannot find name 'so'.
    so.g; // ok
    ~~
!!! error TS2304: Cannot find name 'so'.
    sso.p;
    sso.m(); // ok
    sso.g; // ok
