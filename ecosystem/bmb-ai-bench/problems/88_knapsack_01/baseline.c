#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n, cap; scanf("%d %d", &n, &cap);
    int *wt = malloc(n*sizeof(int)), *val = malloc(n*sizeof(int));
    for (int i = 0; i < n; i++) scanf("%d %d", &wt[i], &val[i]);
    long long *dp = calloc(cap+1, sizeof(long long));
    for (int i = 0; i < n; i++)
        for (int w = cap; w >= wt[i]; w--) {
            long long c = dp[w-wt[i]] + val[i];
            if (c > dp[w]) dp[w] = c;
        }
    printf("%lld\n", dp[cap]);
    free(wt); free(val); free(dp); return 0;
}
