#include <stdio.h>
long long p10(int k) { long long r=1; for(int i=0;i<k;i++) r*=10; return r; }
int find_digit(long long n, int k) {
    long long range_digits = (long long)k * 9 * p10(k-1);
    if (n <= range_digits) {
        long long idx = (n-1)/k;
        long long num = p10(k-1) + idx;
        int pos = (int)((n-1) % k);
        return (int)((num / p10(k-1-pos)) % 10);
    }
    return find_digit(n - range_digits, k+1);
}
int main() {
    long long n; scanf("%lld", &n);
    printf("%d\n", find_digit(n, 1));
    return 0;
}
