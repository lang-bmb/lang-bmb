#include <stdio.h>
#include <stdlib.h>
int main() {
    int n;
    scanf("%d", &n);
    int *arr = (int*)malloc(n*sizeof(int));
    for (int i = 0; i < n; i++) scanf("%d", &arr[i]);
    int best = 1, cur = 1;
    for (int i = 1; i < n; i++) {
        if (arr[i] > arr[i-1]) {
            cur++;
            if (cur > best) best = cur;
        } else {
            cur = 1;
        }
    }
    printf("%d\n", best);
    free(arr);
    return 0;
}
