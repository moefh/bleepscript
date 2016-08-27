
function test1(a, b) {
    printf("inside test1()\n");
    
    var f = function(x) {
      printf("{}, {}, {}\n", a, b, x);
    };
    f(1);
}

function test2() {
    printf("inside test()\n");
    while (true) {
        break;
    }
}
