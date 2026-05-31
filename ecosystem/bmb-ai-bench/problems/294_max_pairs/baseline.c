#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    int freq[101] = {0};
    for (int i = 0; i < n; i++) { int v; scanf("%d", &v); freq[v]++; }
    int pairs = 0, leftovers = 0;
    for (int i = 0; i <= 100; i++) { pairs += freq[i]/2; leftovers += freq[i]%2; }
    printf("%d %d\n", pairs, leftovers);
    return 0;
}
