#include <stdio.h>
#include <stdlib.h>
int main() {
    int n;
    scanf("%d", &n);
    int *arr = (int*)malloc(n*sizeof(int));
    int *cnt = (int*)calloc(n+1, sizeof(int));
    for (int i = 0; i < n; i++) { scanf("%d", &arr[i]); cnt[arr[i]]++; }
    int dup = 0, miss = 0;
    for (int i = 1; i <= n; i++) {
        if (cnt[i] == 2) dup = i;
        if (cnt[i] == 0) miss = i;
    }
    printf("%d %d\n", dup, miss);
    free(arr); free(cnt);
    return 0;
}
