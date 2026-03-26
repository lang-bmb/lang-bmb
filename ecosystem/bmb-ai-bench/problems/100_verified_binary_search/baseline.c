#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n; scanf("%d", &n);
    long long *a = malloc(n*sizeof(long long));
    for(int i=0;i<n;i++) scanf("%lld",&a[i]);
    int q; scanf("%d", &q);
    while(q--) {
        long long t; scanf("%lld", &t);
        int lo=0,hi=n-1,res=-1;
        while(lo<=hi) {
            int mid=lo+(hi-lo)/2;
            if(a[mid]==t){res=mid;break;} else if(a[mid]<t)lo=mid+1; else hi=mid-1;
        }
        printf("%d\n", res);
    }
    free(a); return 0;
}
