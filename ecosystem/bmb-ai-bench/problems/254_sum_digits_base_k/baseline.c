#include <stdio.h>
int main() {
    int n, k;
    scanf("%d", &n);
    scanf("%d", &k);
    int sum = 0;
    while (n > 0) { sum += n % k; n /= k; }
    printf("%d\n", sum);
    return 0;
}
