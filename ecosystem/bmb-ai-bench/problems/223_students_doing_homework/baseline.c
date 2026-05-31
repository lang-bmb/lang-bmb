#include <stdio.h>
#include <stdlib.h>
int main() {
    int n;
    scanf("%d", &n);
    int *s = (int*)malloc(n * sizeof(int));
    int *e = (int*)malloc(n * sizeof(int));
    for (int i = 0; i < n; i++) scanf("%d", &s[i]);
    for (int i = 0; i < n; i++) scanf("%d", &e[i]);
    int t;
    scanf("%d", &t);
    int cnt = 0;
    for (int i = 0; i < n; i++) if (s[i] <= t && t <= e[i]) cnt++;
    printf("%d\n", cnt);
    free(s); free(e);
    return 0;
}
