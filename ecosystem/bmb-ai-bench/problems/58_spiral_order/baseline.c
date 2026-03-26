#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n; scanf("%d", &n);
    long long *m = malloc(n * n * sizeof(long long));
    for (int i = 0; i < n * n; i++) scanf("%lld", &m[i]);
    int top = 0, bot = n-1, left = 0, right = n-1, first = 1;
    while (top <= bot && left <= right) {
        for (int c = left; c <= right; c++) { if (!first) printf(" "); printf("%lld", m[top*n+c]); first = 0; }
        top++;
        for (int r = top; r <= bot; r++) { printf(" "); printf("%lld", m[r*n+right]); }
        right--;
        if (top <= bot) { for (int c = right; c >= left; c--) { printf(" "); printf("%lld", m[bot*n+c]); } bot--; }
        if (left <= right) { for (int r = bot; r >= top; r--) { printf(" "); printf("%lld", m[r*n+left]); } left++; }
    }
    printf("\n");
    free(m);
    return 0;
}
