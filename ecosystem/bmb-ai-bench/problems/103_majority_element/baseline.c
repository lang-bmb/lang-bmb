#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n; scanf("%d", &n);
    long long *a = malloc(n * sizeof(long long));
    for (int i = 0; i < n; i++) scanf("%lld", &a[i]);
    long long candidate = a[0]; int count = 1;
    for (int i = 1; i < n; i++) {
        if (count == 0) { candidate = a[i]; count = 1; }
        else if (a[i] == candidate) count++;
        else count--;
    }
    printf("%lld\n", candidate);
    free(a); return 0;
}
