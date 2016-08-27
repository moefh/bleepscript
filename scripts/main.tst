
include "include.tst"
include "mandelbrot.tst"

function make_counter(start) {
    return function() {
        return start = start + 1;
    };
}

function main(arg) {
    printf("Hello, world!\n");
    #test_function("Hello!");
    printf("Argument from command line: '%s'\n", arg);
    
    var c1 = make_counter(0);
    var c2 = make_counter(10);
    printf("%d, %d\n", c1(), c2());
    printf("%d, %d\n", c1(), c2());

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
    
    return 42;
}
