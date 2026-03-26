#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n; scanf("%d", &n);
    long long *a = malloc(n*sizeof(long long));
    for(int i=0;i<n;i++) scanf("%lld",&a[i]);
    int q; scanf("%d", &q);
    while(q--) { int l,r; scanf("%d %d",&l,&r); long long s=0; for(int i=l;i<=r;i++) s+=a[i]; printf("%lld\n",s); }
    free(a); return 0;
}
