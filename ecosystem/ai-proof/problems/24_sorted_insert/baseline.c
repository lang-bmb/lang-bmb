#include <stdio.h>
#include <stdlib.h>

int main(void) {
    int n;
    scanf("%d", &n);
    long long *arr = (long long *)malloc((n + 1) * sizeof(long long));
    for (int i = 0; i < n; i++) scanf("%lld", &arr[i]);
    long long val;
    scanf("%lld", &val);
    int pos = n;
    for (int i = 0; i < n; i++) {
        if (arr[i] > val) { pos = i; break; }
    }
    for (int i = n; i > pos; i--) arr[i] = arr[i - 1];
    arr[pos] = val;
    for (int i = 0; i <= n; i++) {
        if (i > 0) printf(" ");
        printf("%lld", arr[i]);
    }
    printf("\n");
    free(arr);
    return 0;
}
