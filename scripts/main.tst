
include "include.tst"

function main(args) {
    printf("%d\n", 1+2);
    x = !-a-b*c^d^e;
    x = 1 && 3 || 5 == 7;
    
    1;;;
    
    {{}}
    
    var x = function(a,b) { printf("x\n"); };
    
    {
        printf("another block\n");
    }
}
