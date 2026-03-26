#include <stdio.h>
int main(void) {
    int t; scanf("%d", &t);
    while (t--) { long long n; scanf("%lld", &n); int c=0; while(n>0){c+=n&1;n>>=1;} printf("%d\n",c); }
    return 0;
}
