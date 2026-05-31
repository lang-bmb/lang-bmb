#include <stdio.h>
int main() {
    int n;
    scanf("%d", &n);
    int arr[1001];
    for (int i = 0; i < n; i++) scanf("%d", &arr[i]);
    for (int x = 0; x <= n; x++) {
        int cnt = 0;
        for (int i = 0; i < n; i++) if (arr[i] >= x) cnt++;
        if (cnt == x) { printf("%d\n", x); return 0; }
    }
    printf("-1\n");
    return 0;
}
