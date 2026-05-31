#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    int k; scanf("%d", &k);
    int tickets[100];
    for (int i = 0; i < n; i++) scanf("%d", &tickets[i]);
    int time = 0;
    for (int i = 0; i < n; i++) {
        int cap = (i <= k) ? tickets[k] : tickets[k] - 1;
        int t = tickets[i];
        time += (t < cap) ? t : cap;
    }
    printf("%d\n", time);
    return 0;
}
