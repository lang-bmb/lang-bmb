#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    int nums[1000];
    for (int i = 0; i < n; i++) scanf("%d", &nums[i]);
    int ans = -1, min_so_far = nums[0];
    for (int j = 1; j < n; j++) {
        if (nums[j] > min_so_far) {
            int diff = nums[j] - min_so_far;
            if (diff > ans) ans = diff;
        }
        if (nums[j] < min_so_far) min_so_far = nums[j];
    }
    printf("%d\n", ans);
    return 0;
}
