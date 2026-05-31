#include <stdio.h>
#include <stdlib.h>
int cmp(const void* a, const void* b) { return *(int*)a - *(int*)b; }
int main() {
    int n;
    scanf("%d", &n);
    int arr[10001];
    for (int i = 0; i < n; i++) scanf("%d", &arr[i]);
    qsort(arr, n, sizeof(int), cmp);
    int k = n / 20;
    long long sum = 0;
    for (int i = k; i < n - k; i++) sum += arr[i];
    printf("%lld\n", sum / (n - 2 * k));
    return 0;
}
