#include <stdio.h>
#include <stdlib.h>
int main() {
    int n;
    scanf("%d", &n);
    int *arr = (int*)malloc(n*sizeof(int));
    for (int i = 0; i < n; i++) scanf("%d", &arr[i]);
    if (n < 3) { printf("0\n"); free(arr); return 0; }
    int i = 0;
    while (i + 1 < n && arr[i+1] > arr[i]) i++;
    if (i == 0 || i == n - 1) { printf("0\n"); free(arr); return 0; }
    while (i + 1 < n && arr[i+1] < arr[i]) i++;
    printf("%d\n", i == n - 1 ? 1 : 0);
    free(arr);
    return 0;
}
