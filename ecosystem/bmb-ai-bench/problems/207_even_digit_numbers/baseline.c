#include <stdio.h>
int digit_count(int n) {
    if (n == 0) return 1;
    int c = 0;
    while (n > 0) { c++; n /= 10; }
    return c;
}
int main() {
    int n;
    scanf("%d", &n);
    int count = 0;
    for (int i = 0; i < n; i++) {
        int x;
        scanf("%d", &x);
        if (digit_count(x) % 2 == 0) count++;
    }
    printf("%d\n", count);
    return 0;
}
