#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    int nums[50];
    for (int i = 0; i < n; i++) scanf("%d", &nums[i]);
    int or_val = 0;
    for (int i = 0; i < n; i++) or_val |= nums[i];
    int pow2 = 1;
    for (int i = 0; i < n - 1; i++) pow2 *= 2;
    printf("%d\n", or_val * pow2);
    return 0;
}
