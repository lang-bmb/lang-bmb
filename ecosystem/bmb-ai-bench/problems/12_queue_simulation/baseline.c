#include <stdio.h>
#include <stdlib.h>

int main(void) {
    int q;
    scanf("%d", &q);
    long long *queue = (long long *)malloc(q * sizeof(long long));
    int head = 0, tail = 0;
    for (int i = 0; i < q; i++) {
        int op;
        scanf("%d", &op);
        if (op == 1) {
            long long x;
            scanf("%lld", &x);
            queue[tail++] = x;
        } else {
            if (head < tail) printf("%lld\n", queue[head++]);
            else printf("-1\n");
        }
    }
    free(queue);
    return 0;
}
