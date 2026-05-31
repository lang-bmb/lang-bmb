#include <stdio.h>
int main() {
    int n;
    scanf("%d", &n);
    long long a=0, b=1, c=1;
    for (int i = 0; i < n; i++) {
        long long d = a + b + c;
        a = b; b = c; c = d;
    }
    printf("%lld\n", a);
    return 0;
}
