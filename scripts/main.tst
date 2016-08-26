
include "include.tst"

function main(args) {
    printf("Hello, world!\n");

    var x = 1;
    var set_x = function(val) {
        x = val;
    };
    set_x(2 + 1);
    
    dump_env();
}
