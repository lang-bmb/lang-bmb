#include <stdio.h>
int main() {
    int a, b; scanf("%d %d", &a, &b);
    int r;
    if (a == b) r = 0;
    else if ((a == 0 && b == 2) || (a == 1 && b == 0) || (a == 2 && b == 1)) r = 1;
    else r = 2;
    printf("%d\n", r);
    return 0;
}
