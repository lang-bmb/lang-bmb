#include <stdio.h>
#include <stdlib.h>

int main(void) {
    int cap, q;
    scanf("%d %d", &cap, &q);
    long long *stack = (long long *)malloc(cap * sizeof(long long));
    int size = 0;
    for (int i = 0; i < q; i++) {
        int op;
        scanf("%d", &op);
        if (op == 1) {
            long long x;
            scanf("%lld", &x);
            if (size < cap) stack[size++] = x;
            else printf("FULL\n");
        } else if (op == 2) {
            if (size > 0) printf("%lld\n", stack[--size]);
            else printf("EMPTY\n");
        } else {
            printf("%d\n", size);
        }
    }
    free(stack);
    return 0;
}
