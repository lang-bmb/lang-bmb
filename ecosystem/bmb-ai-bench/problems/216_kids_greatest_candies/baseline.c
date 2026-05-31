#include <stdio.h>
#include <stdlib.h>
int main() {
    int n;
    scanf("%d", &n);
    int *arr = (int*)malloc(n * sizeof(int));
    for (int i = 0; i < n; i++) scanf("%d", &arr[i]);
    int extra;
    scanf("%d", &extra);
    int mx = 0;
    for (int i = 0; i < n; i++) if (arr[i] > mx) mx = arr[i];
    for (int i = 0; i < n; i++)
        printf("%d\n", arr[i] + extra >= mx ? 1 : 0);
    free(arr);
    return 0;
}
