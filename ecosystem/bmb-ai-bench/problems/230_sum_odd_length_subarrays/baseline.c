#include <stdio.h>
int main() {
    int n;
    scanf("%d", &n);
    int arr[1000];
    for (int i = 0; i < n; i++) scanf("%d", &arr[i]);
    int total = 0;
    for (int start = 0; start < n; start++)
        for (int len = 1; start + len <= n; len += 2) {
            int s = 0;
            for (int k = 0; k < len; k++) s += arr[start + k];
            total += s;
        }
    printf("%d\n", total);
    return 0;
}
