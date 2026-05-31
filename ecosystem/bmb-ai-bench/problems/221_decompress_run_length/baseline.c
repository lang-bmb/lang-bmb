#include <stdio.h>
int main() {
    int n;
    scanf("%d", &n);
    for (int i = 0; i < n; i++) {
        int freq, val;
        scanf("%d %d", &freq, &val);
        for (int j = 0; j < freq; j++) printf("%d\n", val);
    }
    return 0;
}
