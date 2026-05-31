#include <stdio.h>
#include <stdlib.h>
int main() {
    int n;
    scanf("%d", &n);
    int *arr = (int*)malloc(n * sizeof(int));
    for (int i = 0; i < n; i++) scanf("%d", &arr[i]);
    long long sum = 0;
    int mx = arr[0], mn = arr[0];
    for (int i = 0; i < n; i++) {
        sum += arr[i];
        if (arr[i] > mx) mx = arr[i];
        if (arr[i] < mn) mn = arr[i];
    }
    printf("%lld\n", (sum - mx - mn) / (n - 2));
    free(arr);
    return 0;
}
