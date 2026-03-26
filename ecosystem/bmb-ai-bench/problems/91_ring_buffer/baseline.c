#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int cap; scanf("%d", &cap);
    long long *buf = calloc(cap, sizeof(long long));
    int head=0, tail=0, size=0;
    int n; scanf("%d", &n);
    while (n--) {
        int op; scanf("%d", &op);
        if (op==1) { long long v; scanf("%lld", &v); buf[tail]=v; tail=(tail+1)%cap; if(size<cap)size++; else head=(head+1)%cap; }
        else if (op==2) { if(size>0) { printf("%lld\n", buf[head]); head=(head+1)%cap; size--; } else printf("-1\n"); }
        else printf("%d\n", size);
    }
    free(buf); return 0;
}
