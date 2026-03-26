#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n; scanf("%d", &n);
    if (n == 0) { printf("0\n"); return 0; }
    long long *a = malloc(n * sizeof(long long));
    for (int i = 0; i < n; i++) scanf("%lld", &a[i]);
    int pairs = 0;
    long long vals[4096], counts[4096];
    long long cur = a[0]; int cnt = 1;
    for (int i = 1; i < n; i++) {
        if (a[i] == cur) cnt++;
        else { vals[pairs] = cur; counts[pairs] = cnt; pairs++; cur = a[i]; cnt = 1; }
    }
    vals[pairs] = cur; counts[pairs] = cnt; pairs++;
    printf("%d\n", pairs);
    for (int i = 0; i < pairs; i++) printf("%lld %d\n", vals[i], (int)counts[i]);
    free(a);
    return 0;
}
