#include <stdio.h>
int is_palindrome(long long n) {
    if (n < 0) return 0;
    long long x = n, rev = 0;
    while (x > 0) { rev = rev * 10 + x % 10; x /= 10; }
    return rev == n;
}
int main(void) {
    int t; scanf("%d", &t);
    while (t--) {
        long long n; scanf("%lld", &n);
        printf("%s\n", is_palindrome(n) ? "yes" : "no");
    }
    return 0;
}
