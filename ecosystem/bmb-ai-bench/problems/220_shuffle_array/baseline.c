#include <stdio.h>
#include <stdlib.h>
int main() {
    int n;
    scanf("%d", &n);
    int *arr = (int*)malloc(2 * n * sizeof(int));
    for (int i = 0; i < 2 * n; i++) scanf("%d", &arr[i]);
    for (int i = 0; i < n; i++) {
        printf("%d\n", arr[i]);
        printf("%d\n", arr[i + n]);
    }
    free(arr);
    return 0;
}
