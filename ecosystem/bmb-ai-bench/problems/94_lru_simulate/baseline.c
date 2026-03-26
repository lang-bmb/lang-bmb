#include <stdio.h>
int main(void) {
    int k, n; scanf("%d %d", &k, &n);
    int cache[100], sz=0, faults=0;
    for (int i = 0; i < n; i++) {
        int page; scanf("%d", &page);
        int found = -1;
        for (int j = 0; j < sz; j++) if (cache[j]==page) { found=j; break; }
        if (found >= 0) {
            for (int j=found; j<sz-1; j++) cache[j]=cache[j+1];
            cache[sz-1]=page;
        } else {
            faults++;
            if (sz >= k) { for(int j=0;j<sz-1;j++) cache[j]=cache[j+1]; cache[sz-1]=page; }
            else cache[sz++]=page;
        }
    }
    printf("%d\n", faults);
    return 0;
}
