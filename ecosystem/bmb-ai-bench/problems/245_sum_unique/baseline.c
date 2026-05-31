#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    int a[100], cnt[101] = {0};
    for (int i = 0; i < n; i++) { scanf("%d", &a[i]); cnt[a[i]]++; }
    int sum = 0;
    for (int i = 0; i < n; i++) if (cnt[a[i]] == 1) sum += a[i];
    printf("%d\n", sum);
    return 0;
}
