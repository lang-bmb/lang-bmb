#include <stdio.h>
int main() {
    int n;
    scanf("%d", &n);
    long long arr[100001];
    for (int i = 0; i < n; i++) scanf("%lld", &arr[i]);
    long long max_reach = 0;
    for (int i = 0; i < n; i++) {
        if (i > max_reach) { printf("false\n"); return 0; }
        long long reach = i + arr[i];
        if (reach > max_reach) max_reach = reach;
    }
    printf("true\n");
    return 0;
}
