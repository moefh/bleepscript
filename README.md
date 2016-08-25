# BleepScript

A simple toy scriting language written in Rust.

The code is in a very early stage, it only parses a subset of the language and dumps the created AST.

Example code:

    function main(args) {
        print("Hello, world!\n");
        f = function(x) {
            print("Hello, ", x, " from anonymous function \n");
        };
        f("world");
    }


