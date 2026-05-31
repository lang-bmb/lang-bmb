#include <stdio.h>
int main() {
    int n;
    scanf("%d", &n);
    int nums[101] = {0};
    if (n >= 1) nums[1] = 1;
    for (int i = 2; i <= n; i++) {
        if (i % 2 == 0) nums[i] = nums[i/2];
        else nums[i] = nums[i/2] + nums[i/2+1];
    }
    int mx = 0;
    for (int i = 0; i <= n; i++) if (nums[i] > mx) mx = nums[i];
    printf("%d\n", mx);
    return 0;
}
