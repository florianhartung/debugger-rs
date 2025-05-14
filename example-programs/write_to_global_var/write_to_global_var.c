// Has to be compiled with the -no-pie option, so the variable always has the same address
// Breaks the example otherwise

#include <stdio.h>

int a = 5;

int before_write() {
    printf("before write: %d\n", a);
}

int after_write() {
    printf("after write: %d\n", a);
}

int main() {
    before_write();
    a = 15;
    after_write();
}
