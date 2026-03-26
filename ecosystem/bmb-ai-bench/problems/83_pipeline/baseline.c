#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n; scanf("%d", &n);
    long long *a = malloc(n * sizeof(long long));
    for (int i = 0; i < n; i++) scanf("%lld", &a[i]);
    int m; scanf("%d", &m);
    while (m--) {
        int op; scanf("%d", &op);
        if (op==1) { long long k; scanf("%lld",&k); for(int i=0;i<n;i++) a[i]+=k; }
        else if (op==2) { long long k; scanf("%lld",&k); for(int i=0;i<n;i++) a[i]*=k; }
        else if (op==3) { for(int i=0;i<n;i++) a[i]=-a[i]; }
        else if (op==4) { for(int i=0;i<n;i++) if(a[i]<0) a[i]=-a[i]; }
        else { for(int l=0,r=n-1;l<r;l++,r--) { long long t=a[l];a[l]=a[r];a[r]=t; } }
    }
    for (int i = 0; i < n; i++) { if(i>0)printf(" "); printf("%lld",a[i]); }
    printf("\n"); free(a); return 0;
}
