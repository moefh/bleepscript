
include "include.tst"
include "mandelbrot.tst"

function map_literal() {
    var x = {
        a : 1,
        b : 2,
        c : make_counter(0),
        "any string" : 42
    };
    printf("%s\n", x);
    printf("x.a = '%s'\n", x.a);
    printf("x.c = '%s'\n", x.c);
    x.c.read();
    printf("printf = %s\n", printf);
}

function make_counter(start) {
    return {
        next : function() {
            return start = start + 1;
        },

        read : function() {
            return start;
        },
    };
}

function main(arg) {
    printf("Hello, world!\n");
    printf("Argument from command line: '%s'\n", arg);

    map_literal();
    
    var c1 = make_counter(0);
    var c2 = make_counter(10);
    c1.next();
    c2.next();
    c1.next();
    c2.next();
    printf("%d, %d\n", c1.read(), c2.read());

    var x = 1;
    if (x == 1) {
        printf("x is one!\n");
    } else {
        printf("x isn't one!\n");
    }

    while (x <= 100) {
        if (x == 6) break;
        printf("%d\n", x);
        x = x + 1;
    }

    var set_x = function(val) {
        x = val;
    };
    set_x(2);
    printf("x is now %d\n", x);

    mandelbrot(-2,-2, 2,2, 76,38, 250);
    #mandelbrot(-2,-2, 2,2, 120,60, 2500);
    
    return 42;
}
