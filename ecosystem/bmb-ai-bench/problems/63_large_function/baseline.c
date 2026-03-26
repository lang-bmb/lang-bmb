#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n; scanf("%d", &n);
    long long *a = malloc(n * sizeof(long long));
    for (int i = 0; i < n; i++) scanf("%lld", &a[i]);
    long long sum=0, mn=a[0], mx=a[0]; int even=0, pos=0;
    for (int i = 0; i < n; i++) {
        sum+=a[i]; if(a[i]<mn)mn=a[i]; if(a[i]>mx)mx=a[i];
        if(a[i]%2==0)even++; if(a[i]>0)pos++;
    }
    printf("%lld\n%lld\n%lld\n%d\n%d\n", sum, mn, mx, even, pos);
    free(a); return 0;
}
