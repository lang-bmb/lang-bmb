#include <stdio.h>
#include <stdlib.h>
int main() {
    int n;
    scanf("%d", &n);
    int *xs = (int*)malloc(n*sizeof(int));
    int *ys = (int*)malloc(n*sizeof(int));
    for (int i = 0; i < n; i++) { scanf("%d", &xs[i]); scanf("%d", &ys[i]); }
    int dx = xs[1]-xs[0], dy = ys[1]-ys[0];
    int ok = 1;
    for (int i = 2; i < n; i++) {
        int cx = xs[i]-xs[0], cy = ys[i]-ys[0];
        if ((long long)dx*cy != (long long)dy*cx) { ok = 0; break; }
    }
    printf("%d\n", ok);
    free(xs); free(ys);
    return 0;
}
