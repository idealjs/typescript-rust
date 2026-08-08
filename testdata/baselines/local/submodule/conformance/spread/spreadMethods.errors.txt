spreadMethods.ts(4,9): error TS1005: ';' expected.
spreadMethods.ts(22,21): error TS2304: Cannot find name 'm'.
spreadMethods.ts(22,22): error TS1005: ',' expected.
spreadMethods.ts(22,22): error TS2304: Cannot find name '('.
spreadMethods.ts(22,23): error TS1005: ',' expected.
spreadMethods.ts(22,23): error TS2304: Cannot find name ')'.
spreadMethods.ts(22,25): error TS1005: ',' expected.
spreadMethods.ts(22,25): error TS2304: Cannot find name '{'.
spreadMethods.ts(22,30): error TS2304: Cannot find name 'get'.
spreadMethods.ts(22,34): error TS1005: ',' expected.
spreadMethods.ts(22,35): error TS1005: ',' expected.
spreadMethods.ts(22,36): error TS1134: Variable declaration expected.


==== spreadMethods.ts (12 errors) ====
    class K {
        p = 12;
        m() { }
        get g() { return 0; }
            ~
!!! error TS1005: ';' expected.
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
!!! error TS2304: Cannot find name 'm'.
                         ~
!!! error TS1005: ',' expected.
                         ~
!!! error TS2304: Cannot find name '('.
                          ~
!!! error TS1005: ',' expected.
                          ~
!!! error TS2304: Cannot find name ')'.
                            ~
!!! error TS1005: ',' expected.
                            ~
!!! error TS2304: Cannot find name '{'.
                                 ~~~
!!! error TS2304: Cannot find name 'get'.
                                     ~
!!! error TS1005: ',' expected.
                                      ~
!!! error TS1005: ',' expected.
                                       ~
!!! error TS1134: Variable declaration expected.
    let si = { ...i };
    let ssi = { ...i, ...i };
    si.p;
    si.m(); // ok
    si.g; // ok
    ssi.p;
    ssi.m(); // ok
    ssi.g; // ok
    
    let o = { p: 12, m() { }, get g() { return 0; } };
    let so = { ...o };
    let sso = { ...o, ...o };
    so.p;
    so.m(); // ok
    so.g; // ok
    sso.p;
    sso.m(); // ok
    sso.g; // ok
