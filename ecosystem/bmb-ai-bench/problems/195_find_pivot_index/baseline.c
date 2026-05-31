#include <stdio.h>
#include <stdlib.h>
int main() {
    int n;
    scanf("%d", &n);
    long long *arr = (long long*)malloc(n*sizeof(long long));
    for (int i = 0; i < n; i++) scanf("%lld", &arr[i]);
    long long total = 0;
    for (int i = 0; i < n; i++) total += arr[i];
    long long left = 0;
    for (int i = 0; i < n; i++) {
        long long right = total - left - arr[i];
        if (left == right) { printf("%d\n", i); free(arr); return 0; }
        left += arr[i];
    }
    printf("-1\n");
    free(arr);
    return 0;
}
