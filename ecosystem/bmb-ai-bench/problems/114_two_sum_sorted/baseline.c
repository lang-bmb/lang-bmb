#include <stdio.h>
int main() {
    int n, target;
    scanf("%d %d", &n, &target);
    long long arr[100001];
    for (int i = 0; i < n; i++) scanf("%lld", &arr[i]);
    int lo = 0, hi = n - 1;
    while (lo < hi) {
        long long sum = arr[lo] + arr[hi];
        if (sum == target) { printf("%d\n%d\n", lo + 1, hi + 1); return 0; }
        else if (sum < target) lo++;
        else hi--;
    }
    return 0;
}
