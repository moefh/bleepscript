
include "include.tst"
include "mandelbrot.tst"

function test_map_literal() {
	printf("\n-> Testing map literal\n");
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
}

function test_vec_literal() {
	printf("\n-> Testing vector literal\n");
    var v = [ 1, 2, 3, "a string", function(arg) { printf("arg is '%s'\n", arg); } ];
    printf("v = %s\n", v);
    printf("v[0] = '%s'\n", v[0]);
    printf("v[3] = '%s'\n", v[3]);
    v[4]("hello");
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

function test_closure() {
	printf("\n-> Testing closure\n");
    var c1 = make_counter(0);
    var c2 = make_counter(10);
    c1.next();
    c2.next();
    c1.next();
    c2.next();
    printf("%d, %d\n", c1.read(), c2.read());
}

function main(arg) {
    printf("\n-> Argument from command line: '%s'\n", arg);

    test_map_literal();
    test_vec_literal();
	test_closure();

    printf("\n-> Testing if\n");
    var x = 1;
    if (x == 1) {
        printf("x is one!\n");
    } else {
        printf("ERROR: x isn't one! (this is not supposed to happen)\n");
    }

    [1,2][0] = 3;

    printf("\n-> Testing while\n");
    while (x <= 100) {
        if (x == 6) break;
        printf("%d\n", x);
        x = x + 1;
    }

    mandelbrot(-2,-2, 2,2, 76,38, 150);
    #mandelbrot(-2,-2, 2,2, 120,60, 2500);
    
    return 42;
}
