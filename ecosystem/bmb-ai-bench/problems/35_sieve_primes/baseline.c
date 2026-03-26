#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n; scanf("%d", &n);
    if (n < 2) { printf("0\n"); return 0; }
    char *sieve = calloc(n + 1, 1);
    for (int i = 2; (long long)i * i <= n; i++)
        if (!sieve[i]) for (int j = i * i; j <= n; j += i) sieve[j] = 1;
    int count = 0;
    for (int i = 2; i <= n; i++) if (!sieve[i]) count++;
    printf("%d\n", count);
    free(sieve);
    return 0;
}
