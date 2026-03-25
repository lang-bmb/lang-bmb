#include <stdio.h>
#include <stdlib.h>

int main(void) {
    int n;
    scanf("%d", &n);
    long long *arr = (long long *)malloc(n * sizeof(long long));
    long long *prefix = (long long *)malloc((n + 1) * sizeof(long long));
    for (int i = 0; i < n; i++) scanf("%lld", &arr[i]);
    prefix[0] = 0;
    for (int i = 0; i < n; i++) prefix[i + 1] = prefix[i] + arr[i];
    int q;
    scanf("%d", &q);
    for (int i = 0; i < q; i++) {
        int l, r;
        scanf("%d %d", &l, &r);
        printf("%lld\n", prefix[r + 1] - prefix[l]);
    }
    free(arr);
    free(prefix);
    return 0;
}
