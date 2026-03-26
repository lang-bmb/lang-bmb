#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int na; scanf("%d", &na);
    long long *a = malloc(na*sizeof(long long));
    for(int i=0;i<na;i++) scanf("%lld",&a[i]);
    int nb; scanf("%d", &nb);
    long long *b = malloc(nb*sizeof(long long));
    for(int i=0;i<nb;i++) scanf("%lld",&b[i]);
    int i=0,j=0;
    while(i<na||j<nb) {
        if(i<na&&(j>=nb||a[i]<=b[j])) { if(i+j>0)printf(" "); printf("%lld",a[i++]); }
        else { if(i+j>0)printf(" "); printf("%lld",b[j++]); }
    }
    printf("\n"); free(a);free(b); return 0;
}
