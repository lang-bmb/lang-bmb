#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int t; scanf("%d", &t);
    while (t--) {
        int n; scanf("%d", &n);
        int *v = malloc(n * sizeof(int));
        for (int i = 0; i < n; i++) scanf("%d", &v[i]);
        int result = 0, i = 0;
        while (i < n) {
            if (i + 1 < n && v[i] < v[i+1]) { result += v[i+1] - v[i]; i += 2; }
            else { result += v[i]; i++; }
        }
        printf("%d\n", result);
        free(v);
    }
    return 0;
}
