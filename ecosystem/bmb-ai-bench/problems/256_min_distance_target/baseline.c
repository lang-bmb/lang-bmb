#include <stdio.h>
int abs_val(int x) { return x < 0 ? -x : x; }
int main() {
    int n; scanf("%d", &n);
    int nums[101];
    for (int i = 0; i < n; i++) scanf("%d", &nums[i]);
    int target, start;
    scanf("%d", &target);
    scanf("%d", &start);
    int min = n;
    for (int i = 0; i < n; i++) {
        if (nums[i] == target) {
            int d = abs_val(i - start);
            if (d < min) min = d;
        }
    }
    printf("%d\n", min);
    return 0;
}
