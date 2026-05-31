#include <stdio.h>
int main() {
    int n;
    scanf("%d", &n);
    int pow = 1, best_pow = 0;
    int tmp = n;
    while (tmp > 0) {
        if (tmp % 10 == 6) best_pow = pow;
        tmp /= 10;
        pow *= 10;
    }
    printf("%d\n", n + 3 * best_pow);
    return 0;
}
