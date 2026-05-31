#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    int nums[1000];
    for (int i = 0; i < n; i++) scanf("%d", &nums[i]);
    int diff; scanf("%d", &diff);
    int cnt = 0;
    for (int i = 0; i < n; i++)
        for (int j = i+1; j < n; j++)
            for (int k = j+1; k < n; k++)
                if (nums[j]-nums[i]==diff && nums[k]-nums[j]==diff) cnt++;
    printf("%d\n", cnt);
    return 0;
}
