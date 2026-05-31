#include <stdio.h>
#include <stdlib.h>
int cmp(const void *a, const void *b) { return *(int*)a - *(int*)b; }
int main() {
    int n; scanf("%d", &n);
    int nums[1000];
    for (int i = 0; i < n; i++) scanf("%d", &nums[i]);
    qsort(nums, n, sizeof(int), cmp);
    printf("%d\n", (nums[n-1] * nums[n-2]) - (nums[0] * nums[1]));
    return 0;
}
