#include <stdio.h>

int fn_a() {
    printf("A\n");
}

int fn_b() {
    printf("B\n");
}

int fn_c() {
    printf("C\n");
}

int main() {
    fn_a();
    fn_b();
    fn_c();
    fn_c();
}