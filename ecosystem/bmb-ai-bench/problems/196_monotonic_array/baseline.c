#include <stdio.h>
#include <stdlib.h>
int main() {
    int n;
    scanf("%d", &n);
    int *arr = (int*)malloc(n*sizeof(int));
    for (int i = 0; i < n; i++) scanf("%d", &arr[i]);
    int inc = 1, dec = 1;
    for (int i = 1; i < n; i++) {
        if (arr[i] > arr[i-1]) dec = 0;
        if (arr[i] < arr[i-1]) inc = 0;
    }
    printf("%d\n", (inc || dec) ? 1 : 0);
    free(arr);
    return 0;
}
