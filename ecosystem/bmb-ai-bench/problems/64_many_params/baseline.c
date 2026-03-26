#include <stdio.h>
int main(void) {
    int t; scanf("%d", &t);
    while (t--) {
        long long a,b,c,d,e,f,w1,w2,w3,w4,w5,w6;
        scanf("%lld%lld%lld%lld%lld%lld%lld%lld%lld%lld%lld%lld",&a,&b,&c,&d,&e,&f,&w1,&w2,&w3,&w4,&w5,&w6);
        printf("%lld\n", a*w1+b*w2+c*w3+d*w4+e*w5+f*w6);
    }
    return 0;
}
