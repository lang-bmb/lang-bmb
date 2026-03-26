#include <stdio.h>
#include <stdlib.h>

int main(void) {
    int rows, cols;
    scanf("%d %d", &rows, &cols);
    long long *m = (long long *)malloc(rows * cols * sizeof(long long));
    for (int i = 0; i < rows * cols; i++) scanf("%lld", &m[i]);
    for (int j = 0; j < cols; j++) {
        for (int i = 0; i < rows; i++) {
            if (i > 0) printf(" ");
            printf("%lld", m[i * cols + j]);
        }
        printf("\n");
    }
    free(m);
    return 0;
}
