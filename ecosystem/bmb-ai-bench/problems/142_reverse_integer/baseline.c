#include <stdio.h>
int main() {
    long long x;
    scanf("%lld", &x);
    int sign = x < 0 ? -1 : 1;
    long long abs_x = x < 0 ? -x : x;
    long long rev = 0;
    while (abs_x != 0) { rev = rev * 10 + abs_x % 10; abs_x /= 10; }
    rev *= sign;
    if (rev > 2147483647LL || rev < -2147483648LL) printf("0\n");
    else printf("%lld\n", rev);
    return 0;
}
