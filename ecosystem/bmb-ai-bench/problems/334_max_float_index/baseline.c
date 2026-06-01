#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    double bestv; int besti = 0;
    scanf("%lf", &bestv);
    for (int i = 1; i < n; i++) {
        double x; scanf("%lf", &x);
        if (x > bestv) { bestv = x; besti = i; }
    }
    printf("%d\n", besti);
    return 0;
}
