#include <stdio.h>
#include <graalvm/llvm/polyglot.h>

void main() {
    int x = 42;
    polyglot_export("x", x);
    int y = polyglot_eval_file("c", "import_x.c");
}
