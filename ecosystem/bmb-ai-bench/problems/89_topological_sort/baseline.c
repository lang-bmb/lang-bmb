#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n, m; scanf("%d %d", &n, &m);
    int *indeg = calloc(n, sizeof(int));
    int *ef = malloc(m*sizeof(int)), *et = malloc(m*sizeof(int));
    for (int i = 0; i < m; i++) { scanf("%d %d", &ef[i], &et[i]); indeg[et[i]]++; }
    int *q = malloc(n*sizeof(int)), front=0, back=0;
    for (int i = 0; i < n; i++) if (indeg[i]==0) q[back++]=i;
    int *res = malloc(n*sizeof(int)), rc=0;
    while (front < back) {
        int u = q[front++]; res[rc++] = u;
        for (int e = 0; e < m; e++) if (ef[e]==u) { if(--indeg[et[e]]==0) q[back++]=et[e]; }
    }
    for (int i = 0; i < rc; i++) { if(i>0) printf(" "); printf("%d", res[i]); }
    printf("\n"); free(indeg);free(ef);free(et);free(q);free(res); return 0;
}
