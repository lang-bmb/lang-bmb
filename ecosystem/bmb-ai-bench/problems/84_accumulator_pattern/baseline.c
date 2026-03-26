#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int t; scanf("%d", &t);
    while (t--) {
        int op, n; scanf("%d %d", &op, &n);
        long long *a = malloc(n * sizeof(long long));
        for (int i = 0; i < n; i++) scanf("%lld", &a[i]);
        long long acc = a[0];
        for (int i = 1; i < n; i++) {
            if (op==1) acc+=a[i]; else if (op==2) acc*=a[i];
            else if (op==3) { if(a[i]<acc)acc=a[i]; }
            else { if(a[i]>acc)acc=a[i]; }
        }
        printf("%lld\n", acc);
        free(a);
    }
    return 0;
}
