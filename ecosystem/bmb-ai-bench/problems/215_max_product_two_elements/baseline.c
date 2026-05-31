#include <stdio.h>
#include <stdlib.h>
int main() {
    int n;
    scanf("%d", &n);
    int *arr = (int*)malloc(n * sizeof(int));
    for (int i = 0; i < n; i++) scanf("%d", &arr[i]);
    int m1 = 0, m2 = 0;
    for (int i = 0; i < n; i++) {
        if (arr[i] > m1) { m2 = m1; m1 = arr[i]; }
        else if (arr[i] > m2) m2 = arr[i];
    }
    printf("%d\n", (m1 - 1) * (m2 - 1));
    free(arr);
    return 0;
}
