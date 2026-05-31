#include <stdio.h>

int main() {
    int n;
    scanf("%d", &n);
    int best = 0, cur = 0;
    for (int i = 0; i < n; i++) {
        int x;
        scanf("%d", &x);
        if (x == 1) { cur++; if (cur > best) best = cur; }
        else cur = 0;
    }
    printf("%d\n", best);
    return 0;
}
