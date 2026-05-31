#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    int nums[1000];
    for (int i = 0; i < n; i++) scanf("%d", &nums[i]);
    for (int r = 0; r < 2; r++) {
        for (int i = 0; i < n; i++) {
            if (r > 0 || i > 0) printf(" ");
            printf("%d", nums[i]);
        }
    }
    printf("\n");
    return 0;
}
