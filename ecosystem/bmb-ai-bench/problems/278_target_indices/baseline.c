#include <stdio.h>
#include <stdlib.h>
int cmp(const void *a, const void *b) { return *(int*)a - *(int*)b; }
int main() {
    int n; scanf("%d", &n);
    int target; scanf("%d", &target);
    int nums[100], cnt = 0, less_cnt = 0;
    for (int i = 0; i < n; i++) scanf("%d", &nums[i]);
    for (int i = 0; i < n; i++) {
        if (nums[i] < target) less_cnt++;
        if (nums[i] == target) cnt++;
    }
    int first = 1;
    for (int i = 0; i < cnt; i++) {
        if (!first) printf(" ");
        printf("%d", less_cnt + i);
        first = 0;
    }
    printf("\n");
    return 0;
}
