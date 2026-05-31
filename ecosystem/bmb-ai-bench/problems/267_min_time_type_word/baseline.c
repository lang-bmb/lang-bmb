#include <stdio.h>
#include <string.h>
int abs_val(int x) { return x < 0 ? -x : x; }
int min_val(int a, int b) { return a < b ? a : b; }
int main() {
    char w[200];
    scanf("%s", w);
    int n = strlen(w), pos = 0, total = 0;
    for (int i = 0; i < n; i++) {
        int c = w[i] - 'a';
        int diff = abs_val(c - pos);
        total += min_val(diff, 26 - diff) + 1;
        pos = c;
    }
    printf("%d\n", total);
    return 0;
}
