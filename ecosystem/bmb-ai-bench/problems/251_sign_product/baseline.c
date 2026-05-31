#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    int sign = 1;
    for (int i = 0; i < n; i++) {
        int x; scanf("%d", &x);
        if (x == 0) { sign = 0; break; }
        if (x < 0) sign = -sign;
    }
    printf("%d\n", sign);
    return 0;
}
