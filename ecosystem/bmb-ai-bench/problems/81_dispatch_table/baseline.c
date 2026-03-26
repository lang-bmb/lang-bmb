#include <stdio.h>
int main(void) {
    int t; scanf("%d", &t);
    while (t--) {
        int op; long long a, b; scanf("%d %lld %lld", &op, &a, &b);
        long long r;
        if (op==1) r=a+b; else if (op==2) r=a-b; else if (op==3) r=a*b; else r=a>b?a:b;
        printf("%lld\n", r);
    }
    return 0;
}
