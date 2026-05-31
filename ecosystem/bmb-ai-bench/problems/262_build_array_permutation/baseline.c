#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    int nums[1000];
    for (int i = 0; i < n; i++) scanf("%d", &nums[i]);
    for (int i = 0; i < n; i++) {
        if (i > 0) printf(" ");
        printf("%d", nums[nums[i]]);
    }
    printf("\n");
    return 0;
}
