#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    int freq[101] = {0};
    for (int i = 0; i < n; i++) { int v; scanf("%d", &v); freq[v] = 1; }
    int cnt = 0;
    for (int i = 1; i <= 100; i++) if (freq[i]) cnt++;
    printf("%d\n", cnt);
    return 0;
}
