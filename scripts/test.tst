
function main(x) {
    var x = [1,2];
    x[2] = 3;
    printf("%s\n", x);
    
    var y = {"a" : x, "b" : 2, "c" : "banana" };
    printf("before: %s\n", y);
    y.a[1] = 0;
    y.b = 3;
    printf("after: %s\n", y);
    y.goober = function() {};
    y.bla;
    printf("and then: %s\n", y);
    printf("x: %s\n", x);
    return 42;
}
