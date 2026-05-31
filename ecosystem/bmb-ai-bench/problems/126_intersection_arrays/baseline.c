#include <stdio.h>
#include <stdlib.h>

int cmp(const void *a, const void *b) { return *(int*)a - *(int*)b; }

int main() {
    int n1, n2;
    scanf("%d", &n1);
    int *a = malloc(n1 * sizeof(int));
    for (int i = 0; i < n1; i++) scanf("%d", &a[i]);
    scanf("%d", &n2);
    int *b = malloc(n2 * sizeof(int));
    for (int i = 0; i < n2; i++) scanf("%d", &b[i]);
    qsort(a, n1, sizeof(int), cmp);
    qsort(b, n2, sizeof(int), cmp);
    int i = 0, j = 0;
    while (i < n1 && j < n2) {
        if (a[i] == b[j]) { printf("%d\n", a[i]); i++; j++; }
        else if (a[i] < b[j]) i++;
        else j++;
    }
    free(a); free(b);
    return 0;
}
