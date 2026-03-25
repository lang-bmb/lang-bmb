#include <stdio.h>
#include <stdlib.h>

int main(void) {
    int q;
    scanf("%d", &q);
    long long *stack = (long long *)malloc(q * sizeof(long long));
    int top = -1;
    for (int i = 0; i < q; i++) {
        int op;
        scanf("%d", &op);
        if (op == 1) {
            long long x;
            scanf("%lld", &x);
            stack[++top] = x;
        } else if (op == 2) {
            if (top >= 0) printf("%lld\n", stack[top--]);
            else printf("-1\n");
        } else {
            if (top >= 0) printf("%lld\n", stack[top]);
            else printf("-1\n");
        }
    }
    free(stack);
    return 0;
}
