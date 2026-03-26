#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
    int k, n;
    scanf("%d %d", &k, &n);
    int *counts = (int *)calloc(k, sizeof(int));
    for (int i = 0; i < n; i++) {
        int x;
        scanf("%d", &x);
        counts[x]++;
    }
    for (int i = 0; i < k; i++)
        printf("%d %d\n", i, counts[i]);
    free(counts);
    return 0;
}
