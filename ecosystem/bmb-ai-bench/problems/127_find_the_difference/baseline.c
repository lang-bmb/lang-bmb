#include <stdio.h>

int main() {
    long long n;
    scanf("%lld", &n);
    int count = 0;
    while (n > 0) {
        if (n % 2 == 0) n /= 2;
        else n--;
        count++;
    }
    printf("%d\n", count);
    return 0;
}
