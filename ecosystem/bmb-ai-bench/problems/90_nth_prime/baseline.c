#include <stdio.h>
#include <stdlib.h>
#include <string.h>
int main(void) {
    int limit = 8000;
    char *sieve = calloc(limit+1, 1);
    for (int i = 2; (long long)i*i <= limit; i++)
        if (!sieve[i]) for (int j=i*i; j<=limit; j+=i) sieve[j]=1;
    int *primes = malloc(1100*sizeof(int)); int pc=0;
    for (int i = 2; i <= limit; i++) if (!sieve[i]) primes[pc++]=i;
    int t; scanf("%d", &t);
    while (t--) { int n; scanf("%d", &n); printf("%d\n", primes[n-1]); }
    free(sieve); free(primes); return 0;
}
