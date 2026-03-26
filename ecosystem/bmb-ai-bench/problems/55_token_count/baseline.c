#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n; scanf("%d", &n);
    long long *a = malloc(n * sizeof(long long));
    for (int i = 0; i < n; i++) scanf("%lld", &a[i]);
    int count = 0;
    for (int i = 0; i < n; i++) {
        int found = 0;
        for (int j = 0; j < i; j++) if (a[j] == a[i]) { found = 1; break; }
        if (!found) count++;
    }
    printf("%d\n", count);
    free(a);
    return 0;
}
