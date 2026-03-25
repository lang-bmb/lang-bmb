#include <stdio.h>
#include <stdlib.h>

int main(void) {
    int n;
    scanf("%d", &n);
    long long *arr = (long long *)malloc(n * sizeof(long long));
    for (int i = 0; i < n; i++) scanf("%lld", &arr[i]);
    long long max_sum = arr[0], cur_sum = arr[0];
    for (int i = 1; i < n; i++) {
        if (cur_sum + arr[i] > arr[i]) cur_sum = cur_sum + arr[i];
        else cur_sum = arr[i];
        if (cur_sum > max_sum) max_sum = cur_sum;
    }
    printf("%lld\n", max_sum);
    free(arr);
    return 0;
}
