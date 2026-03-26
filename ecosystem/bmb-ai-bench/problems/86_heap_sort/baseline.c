#include <stdio.h>
#include <stdlib.h>
void heapify(long long *a, int n, int root) {
    int largest = root, l = 2*root+1, r = 2*root+2;
    if (l < n && a[l] > a[largest]) largest = l;
    if (r < n && a[r] > a[largest]) largest = r;
    if (largest != root) { long long t=a[root]; a[root]=a[largest]; a[largest]=t; heapify(a,n,largest); }
}
int main(void) {
    int n; scanf("%d", &n);
    long long *a = malloc(n * sizeof(long long));
    for (int i = 0; i < n; i++) scanf("%lld", &a[i]);
    for (int i = n/2-1; i >= 0; i--) heapify(a, n, i);
    for (int i = n-1; i > 0; i--) { long long t=a[0]; a[0]=a[i]; a[i]=t; heapify(a,i,0); }
    for (int i = 0; i < n; i++) { if(i>0) printf(" "); printf("%lld", a[i]); }
    printf("\n"); free(a); return 0;
}
