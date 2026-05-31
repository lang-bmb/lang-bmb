#include <stdio.h>
int main() {
    int m, n, k;
    scanf("%d\n%d\n%d\n", &m, &n, &k);
    int min_r = m, min_c = n;
    for (int i = 0; i < k; i++) {
        int r, c;
        scanf("%d\n%d\n", &r, &c);
        if (r < min_r) min_r = r;
        if (c < min_c) min_c = c;
    }
    printf("%d\n", min_r * min_c);
    return 0;
}
