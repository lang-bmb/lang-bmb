#include <stdio.h>
int main() {
    int n;
    scanf("%d", &n);
    int arr[1001];
    for (int i = 0; i < n; i++) scanf("%d", &arr[i]);
    int k;
    scanf("%d", &k);
    int candidate = 1, missing = 0;
    while (1) {
        int found = 0;
        for (int i = 0; i < n; i++) if (arr[i] == candidate) { found = 1; break; }
        if (!found) missing++;
        if (missing == k) { printf("%d\n", candidate); return 0; }
        candidate++;
    }
    return 0;
}
