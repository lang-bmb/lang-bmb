#include <stdio.h>
#include <string.h>
int main(void) {
    int n; scanf("%d", &n);
    int freq[256]; memset(freq, 0, sizeof(freq));
    for (int i = 0; i < n; i++) { int v; scanf("%d", &v); freq[v]++; }
    int count = 0;
    for (int i = 0; i < 256; i++) if (freq[i] > 0) count++;
    printf("%d\n", count);
    for (int i = 0; i < 256; i++) if (freq[i] > 0) printf("%d %d\n", i, freq[i]);
    return 0;
}
