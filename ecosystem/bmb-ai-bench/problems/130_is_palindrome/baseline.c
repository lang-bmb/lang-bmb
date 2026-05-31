#include <stdio.h>

int main() {
    long long n;
    scanf("%lld", &n);
    if (n < 0) { printf("false\n"); return 0; }
    long long orig = n, rev = 0;
    while (n > 0) { rev = rev * 10 + n % 10; n /= 10; }
    printf("%s\n", orig == rev ? "true" : "false");
    return 0;
}
