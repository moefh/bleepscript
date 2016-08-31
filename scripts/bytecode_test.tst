
function func(arg, x1, x2, x3, x4) {
    var x = "a string";
    x = "another string";
    printf("x = %s\n", arg[x2]);
    printf("args = %s, %s, %s, %s, %s\n", x1, x2, x3, x4, (x2 + x4) / 3);
}

function main(arg) {
    printf("Hello, world!\n");
    printf("Argument from command line: '%s'\n", arg);
	func(arg, "some arg", 1, "another arg", 3);
}
