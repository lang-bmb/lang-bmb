#include <stdio.h>
int main(void) {
    int cap; scanf("%d", &cap);
    long long q[1024]; int front=0,back=0,size=0;
    int n; scanf("%d", &n);
    while(n--) {
        int op; scanf("%d", &op);
        if(op==1) { long long v; scanf("%lld",&v); if(size<cap){q[back++]=v;size++;} else printf("-1\n"); }
        else if(op==2) { if(size>0){printf("%lld\n",q[front++]);size--;} else printf("-1\n"); }
        else printf("%d\n",size);
    }
    return 0;
}
