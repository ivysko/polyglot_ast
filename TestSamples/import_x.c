#include <stdio.h>
#include <graalvm/llvm/polyglot.h>

int main() {
    int y = polyglot_import("x");

    return y;
}
