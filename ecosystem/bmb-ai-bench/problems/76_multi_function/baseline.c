#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n; scanf("%d", &n);
    long long *a = malloc(n * sizeof(long long));
    for (int i = 0; i < n; i++) scanf("%lld", &a[i]);
    long long mn=a[0],mx=a[0],s=0;
    for (int i = 0; i < n; i++) { s+=a[i]; if(a[i]<mn)mn=a[i]; if(a[i]>mx)mx=a[i]; }
    printf("%lld\n%lld\n%lld\n%lld\n%lld\n", s, mn, mx, mn<0?-mn:mn, s>0?1LL:s<0?-1LL:0LL);
    free(a); return 0;
}
