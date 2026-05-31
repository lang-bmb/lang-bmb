#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    int nums[1000];
    for (int i = 0; i < n; i++) scanf("%d", &nums[i]);
    for (int i = 0; i + 2 < n; i++) {
        if (nums[i] + nums[i+1] == nums[i+1] + nums[i+2]) { printf("1\n"); return 0; }
    }
    printf("0\n");
    return 0;
}
