#include <stdio.h>

long long digit_sq_sum(long long n) {
    long long s = 0;
    while (n > 0) { long long d = n % 10; s += d*d; n /= 10; }
    return s;
}

int is_happy(long long n) {
    long long seen[200];
    int cnt = 0;
    while (n != 1 && cnt < 100) {
        for (int i = 0; i < cnt; i++) if (seen[i] == n) return 0;
        seen[cnt++] = n;
        n = digit_sq_sum(n);
    }
    return n == 1;
}

int main() {
    long long n;
    scanf("%lld", &n);
    printf("%s\n", is_happy(n) ? "true" : "false");
    return 0;
}
