#include <stdio.h>
#include <stdlib.h>
int main() {
    int k, n;
    scanf("%d\n%d\n", &k, &n);
    int *bed = (int*)malloc(k*sizeof(int));
    for (int i = 0; i < k; i++) scanf("%d\n", &bed[i]);
    int placed = 0;
    for (int i = 0; i < k && placed < n; i++) {
        int prev = (i == 0) ? 0 : bed[i-1];
        int next = (i == k-1) ? 0 : bed[i+1];
        if (bed[i] == 0 && prev == 0 && next == 0) {
            bed[i] = 1;
            placed++;
        }
    }
    printf("%d\n", placed >= n ? 1 : 0);
    free(bed);
    return 0;
}
