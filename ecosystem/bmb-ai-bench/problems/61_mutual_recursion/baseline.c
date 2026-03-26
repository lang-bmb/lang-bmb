#include <stdio.h>
int is_odd(int n);
int is_even(int n) { if (n == 0) return 1; if (n < 0) return is_even(-n); return is_odd(n-1); }
int is_odd(int n) { if (n == 0) return 0; if (n < 0) return is_odd(-n); return is_even(n-1); }
int main(void) {
    int t; scanf("%d", &t);
    while (t--) { int n; scanf("%d", &n); printf("%d\n", is_even(n)); }
    return 0;
}
