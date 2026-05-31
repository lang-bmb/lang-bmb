#include <stdio.h>
int main() {
    int n;
    scanf("%d", &n);
    long long arr[100001];
    for (int i = 0; i < n; i++) scanf("%lld", &arr[i]);
    int lo = 0, hi = n - 1;
    long long lo_max = 0, hi_max = 0, total = 0;
    while (lo < hi) {
        if (arr[lo] <= arr[hi]) {
            if (arr[lo] > lo_max) lo_max = arr[lo];
            else total += lo_max - arr[lo];
            lo++;
        } else {
            if (arr[hi] > hi_max) hi_max = arr[hi];
            else total += hi_max - arr[hi];
            hi--;
        }
    }
    printf("%lld\n", total);
    return 0;
}
