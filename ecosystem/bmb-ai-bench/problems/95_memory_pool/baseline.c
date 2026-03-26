#include <stdio.h>
int main(void) {
    int sizes[1024], active[1024], nid=0;
    int n; scanf("%d", &n);
    while (n--) {
        int op; scanf("%d", &op);
        if (op==1) { int sz; scanf("%d", &sz); sizes[nid]=sz; active[nid]=1; printf("%d\n", nid++); }
        else if (op==2) { int id; scanf("%d", &id); active[id]=0; }
        else { int total=0,blocks=0; for(int j=0;j<nid;j++) if(active[j]){total+=sizes[j];blocks++;} printf("%d %d\n",total,blocks); }
    }
    return 0;
}
