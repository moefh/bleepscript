
include "include.tst"

function main(args) {
    printf("Hello, world!\n");

    var x;
    if (x == 1) {
        printf("x is one!\n");
    } else {
        printf("x isn't one!\n");
    }

    var set_x = function(val) {
        x = val;
    };
    set_x(2);
    printf("x is %d\n", x);
}
