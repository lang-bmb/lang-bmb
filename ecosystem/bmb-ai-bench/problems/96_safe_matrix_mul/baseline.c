#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n; scanf("%d", &n);
    long long *a=malloc(n*n*sizeof(long long)),*b=malloc(n*n*sizeof(long long)),*c=calloc(n*n,sizeof(long long));
    for(int i=0;i<n*n;i++) scanf("%lld",&a[i]);
    for(int i=0;i<n*n;i++) scanf("%lld",&b[i]);
    for(int i=0;i<n;i++) for(int k=0;k<n;k++) for(int j=0;j<n;j++) c[i*n+j]+=a[i*n+k]*b[k*n+j];
    for(int i=0;i<n;i++){for(int j=0;j<n;j++){if(j>0)printf(" ");printf("%lld",c[i*n+j]);}printf("\n");}
    free(a);free(b);free(c); return 0;
}
