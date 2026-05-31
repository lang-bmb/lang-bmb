#include <stdio.h>
int main() {
    long long n, dur; scanf("%lld %lld", &n, &dur);
    long long *ts = (long long*)__builtin_alloca(n * sizeof(long long));
    for (long long i = 0; i < n; i++) scanf("%lld", &ts[i]);
    long long total = 0;
    for (long long i = 0; i < n - 1; i++) {
        long long gap = ts[i+1] - ts[i];
        total += gap < dur ? gap : dur;
    }
    total += dur;
    printf("%lld\n", total);
    return 0;
}
