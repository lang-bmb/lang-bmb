#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n; scanf("%d", &n);
    long long *a = malloc(n * sizeof(long long));
    int *dp = malloc(n * sizeof(int));
    for (int i = 0; i < n; i++) { scanf("%lld", &a[i]); dp[i] = 1; }
    int mx = 1;
    for (int i = 1; i < n; i++) {
        for (int j = 0; j < i; j++)
            if (a[j] < a[i] && dp[j]+1 > dp[i]) dp[i] = dp[j]+1;
        if (dp[i] > mx) mx = dp[i];
    }
    printf("%d\n", mx); free(a); free(dp); return 0;
}
