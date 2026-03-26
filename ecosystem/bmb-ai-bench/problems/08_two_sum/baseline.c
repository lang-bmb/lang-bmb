#include <stdio.h>
#include <stdlib.h>

int main(void) {
    long long target;
    int n;
    scanf("%lld %d", &target, &n);
    long long *arr = (long long *)malloc(n * sizeof(long long));
    for (int i = 0; i < n; i++) scanf("%lld", &arr[i]);
    for (int i = 0; i < n; i++)
        for (int j = i + 1; j < n; j++)
            if (arr[i] + arr[j] == target) {
                printf("%d %d\n", i, j);
                free(arr);
                return 0;
            }
    free(arr);
    return 0;
}
