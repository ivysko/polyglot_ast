#include <stdio.h>
#include <graalvm/llvm/polyglot.h>

int8_t main() {
    int8_t y = polyglot_as_i8(polyglot_import("x"));

    return y;
}
