
include "mandelbrot.tst"

function func(arg, x1, x2, x3, x4) {
    var x = "a string";
    x = "another string";
    printf("x = %s\n", arg.test);
    printf("args = %s, %s, %s, %s, %s\n", x1, x2, x3, x4, (x2 + x4) / 3);
}

function test_while() {
    var i = 0;
    while (i < 10) {
        printf("%d\n", i);
        i = i + 1;
    }
    return 32;
}

function main(arg) {
    printf("Hello, world!\n");
    printf("Argument from command line: '%s'\n", arg);
	func(arg, "some arg", 1, "another arg", 3);
	printf("test_while() returns %s\n", test_while());
    mandelbrot(-2,-2, 2,2, 76,38, 250);
}
