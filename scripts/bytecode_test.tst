
function func(x1, x2, x3, x4) {
    var x = "a string";
    x = "another string";
    printf("x=%s\n", x);
    printf("args=%s, %s, %s, %s\n", x1, x2, x3, x4);
}

function main(arg) {
    printf("Hello, world!\n");
    printf("Argument from command line: '%s'\n", arg);
	func("some arg", 1, "another arg", 3);
}
