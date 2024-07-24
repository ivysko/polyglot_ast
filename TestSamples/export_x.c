#include <stdio.h>
#include <graalvm/llvm/polyglot.h>

void main() {
    int8_t x = 42;
    polyglot_export("x", x);
    void* evaluated = polyglot_eval_file("llvm", "hello.ll");

    int8_t y = polyglot_as_i8(evaluated);
}
