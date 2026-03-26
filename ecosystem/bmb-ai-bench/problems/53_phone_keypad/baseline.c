#include <stdio.h>
int presses(int d) { return (d == 7 || d == 9) ? 4 : 3; }
int main(void) {
    int n; scanf("%d", &n);
    int total = 0;
    for (int i = 0; i < n; i++) { int d; scanf("%d", &d); total += presses(d); }
    printf("%d\n", total);
    return 0;
}
