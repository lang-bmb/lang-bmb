#include <stdio.h>
int main(void) {
    int t; scanf("%d", &t);
    while (t--) { int n; scanf("%d", &n); printf("%d\n", (n % 2 == 0) ? 0 : 1); }
    return 0;
}
