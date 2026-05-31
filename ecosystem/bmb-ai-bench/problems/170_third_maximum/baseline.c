#include <stdio.h>
#define NONE (-4611686018427387904LL)
int main() {
    long long n; scanf("%lld", &n);
    long long m1=NONE, m2=NONE, m3=NONE;
    for (long long i = 0; i < n; i++) {
        long long x; scanf("%lld", &x);
        if (x==m1||x==m2||x==m3) continue;
        if (x>m1) { m3=m2; m2=m1; m1=x; }
        else if (m2==NONE||x>m2) { m3=m2; m2=x; }
        else if (m3==NONE||x>m3) { m3=x; }
    }
    printf("%lld\n", m3==NONE ? m1 : m3);
    return 0;
}
