#include <stdio.h>
int main(void) {
    int n; scanf("%d", &n);
    for (int i = 0; i < n; i++) {
        int fields; scanf("%d", &fields);
        long long sum = 0;
        for (int j = 0; j < fields; j++) { long long v; scanf("%lld", &v); sum += v; }
        printf("%d %lld\n", fields, sum);
    }
    return 0;
}
