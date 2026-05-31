#include <stdio.h>

int main() {
    unsigned long long n;
    scanf("%llu", &n);
    unsigned int x = (unsigned int)n;
    unsigned int result = 0;
    for (int i = 0; i < 32; i++) {
        result = (result << 1) | (x & 1);
        x >>= 1;
    }
    printf("%u\n", result);
    return 0;
}
