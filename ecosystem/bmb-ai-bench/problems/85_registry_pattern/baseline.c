#include <stdio.h>
int main(void) {
    long long keys[1024], vals[1024]; int sz = 0;
    int n; scanf("%d", &n);
    for (int i = 0; i < n; i++) {
        int op; scanf("%d", &op);
        if (op == 1) {
            long long k, v; scanf("%lld %lld", &k, &v);
            int found = 0;
            for (int j = 0; j < sz; j++) if (keys[j] == k) { vals[j] = v; found = 1; break; }
            if (!found) { keys[sz] = k; vals[sz] = v; sz++; }
        } else if (op == 2) {
            long long k; scanf("%lld", &k);
            int found = 0;
            for (int j = 0; j < sz; j++) if (keys[j] == k) { printf("%lld\n", vals[j]); found = 1; break; }
            if (!found) printf("-1\n");
        } else printf("%d\n", sz);
    }
    return 0;
}
