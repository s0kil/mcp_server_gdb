#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int global_var = 42;
int global_array[5] = {10, 20, 30, 40, 50};

typedef struct {
    int x;
    int y;
    char name[32];
} Point;

int add(int a, int b) {
    int result = a + b;
    return result;
}

int factorial(int n) {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}

void modify_memory(int *ptr) {
    *ptr = 99;
}

void loop_function(void) {
    int sum = 0;
    for (int i = 0; i < 10; i++) {
        sum += i;
    }
    printf("sum = %d\n", sum);
}

int main(int argc, char *argv[]) {
    int a = 10;
    int b = 20;
    int c = add(a, b);
    printf("add: %d\n", c);

    Point p = {.x = 1, .y = 2, .name = "test"};
    printf("point: (%d, %d, %s)\n", p.x, p.y, p.name);

    int fact = factorial(5);
    printf("factorial: %d\n", fact);

    int val = 0;
    modify_memory(&val);
    printf("modified: %d\n", val);

    loop_function();

    for (int i = 0; i < argc; i++) {
        printf("arg[%d] = %s\n", i, argv[i]);
    }

    return 0;
}
