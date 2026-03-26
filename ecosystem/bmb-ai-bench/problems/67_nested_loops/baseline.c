#include <stdio.h>
int main(void) {
    int t; scanf("%d", &t);
    while (t--) {
        int n, target; scanf("%d %d", &n, &target);
        int count = 0;
        for (int i = 0; i < n; i++)
            for (int j = 0; j < n; j++) {
                int k = target - i - j;
                if (k >= 0 && k < n) count++;
            }
        printf("%d\n", count);
    }
    return 0;
}
