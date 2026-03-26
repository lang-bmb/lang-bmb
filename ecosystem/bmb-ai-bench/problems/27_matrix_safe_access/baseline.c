#include <stdio.h>
#include <stdlib.h>

int main(void) {
    int rows, cols;
    scanf("%d %d", &rows, &cols);
    long long *m = (long long *)malloc(rows * cols * sizeof(long long));
    for (int i = 0; i < rows * cols; i++) scanf("%lld", &m[i]);
    int q;
    scanf("%d", &q);
    for (int i = 0; i < q; i++) {
        int r, c;
        scanf("%d %d", &r, &c);
        printf("%lld\n", m[r * cols + c]);
    }
    free(m);
    return 0;
}
