
include "include.tst"

function main(args) {
    printf("Hello, world!\n");

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
}
