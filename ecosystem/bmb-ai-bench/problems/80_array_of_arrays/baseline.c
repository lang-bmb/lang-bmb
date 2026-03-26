#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int r, c; scanf("%d %d", &r, &c);
    long long *v = malloc(r * c * sizeof(long long));
    for (int i = 0; i < r * c; i++) scanf("%lld", &v[i]);
    for (int j = 0; j < c; j++) {
        long long sum = 0;
        for (int i = 0; i < r; i++) sum += v[i * c + j];
        if (j > 0) printf(" ");
        printf("%lld", sum);
    }
    printf("\n");
    free(v); return 0;
}
