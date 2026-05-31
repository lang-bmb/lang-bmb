#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n; scanf("%d", &n);
    long long *a = malloc(n * sizeof(long long));
    for (int i = 0; i < n; i++) scanf("%lld", &a[i]);
    long long min_p = a[0], profit = 0;
    for (int i = 1; i < n; i++) {
        if (a[i] - min_p > profit) profit = a[i] - min_p;
        if (a[i] < min_p) min_p = a[i];
    }
    printf("%lld\n", profit);
    free(a); return 0;
}
