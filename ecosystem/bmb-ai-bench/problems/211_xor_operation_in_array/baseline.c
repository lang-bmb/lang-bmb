#include <stdio.h>
int main() {
    int n, start;
    scanf("%d %d", &n, &start);
    int result = 0;
    for (int i = 0; i < n; i++) result ^= (start + 2 * i);
    printf("%d\n", result);
    return 0;
}
