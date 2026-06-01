#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    int pos = 0, neg = 0, x;
    for (int i = 0; i < n; i++) { scanf("%d", &x); if (x > 0) pos++; else if (x < 0) neg++; }
    printf("%d\n", pos > neg ? pos : neg);
    return 0;
}
