#include <stdio.h>
int main() {
    int n;
    scanf("%d", &n);
    long long arr[10001];
    for (int i = 0; i < n; i++) scanf("%lld", &arr[i]);
    long long cur_max = arr[0], cur_min = arr[0], best = arr[0];
    for (int i = 1; i < n; i++) {
        long long x = arr[i];
        long long a = cur_max * x, b = cur_min * x;
        cur_max = (x > a ? x : a) > b ? (x > a ? x : a) : b;
        cur_min = (x < a ? x : a) < b ? (x < a ? x : a) : b;
        best = best > cur_max ? best : cur_max;
    }
    printf("%lld\n", best);
    return 0;
}
