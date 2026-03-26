#include <stdio.h>
int main(void) {
    int n; scanf("%d", &n);
    long long stack[1024]; int top = 0;
    for (int i = 0; i < n; i++) {
        int op; scanf("%d", &op);
        if (op == 0) { long long val; scanf("%lld", &val); stack[top++] = val; }
        else {
            long long b = stack[--top], a = stack[--top];
            if (op == 1) stack[top++] = a + b;
            else if (op == 2) stack[top++] = a - b;
            else if (op == 3) stack[top++] = a * b;
            else stack[top++] = a / b;
        }
    }
    printf("%lld\n", stack[top - 1]);
    return 0;
}
