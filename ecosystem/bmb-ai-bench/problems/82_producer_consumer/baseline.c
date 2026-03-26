#include <stdio.h>
int main(void) {
    int n; scanf("%d", &n);
    long long queue[4096]; int front = 0, back = 0;
    for (int i = 0; i < n; i++) {
        int op; scanf("%d", &op);
        if (op == 1) { long long v; scanf("%lld", &v); queue[back++] = v; }
        else printf("%lld\n", queue[front++]);
    }
    return 0;
}
