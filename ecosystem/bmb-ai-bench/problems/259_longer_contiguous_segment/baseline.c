#include <stdio.h>
#include <string.h>
int max_run(const char *s, int n, char c) {
    int best = 0, cur = 0;
    for (int i = 0; i < n; i++) {
        if (s[i] == c) { cur++; if (cur > best) best = cur; }
        else cur = 0;
    }
    return best;
}
int main() {
    char s[200];
    scanf("%s", s);
    int n = strlen(s);
    int ones = max_run(s, n, '1');
    int zeros = max_run(s, n, '0');
    printf("%d\n", ones > zeros ? 1 : 0);
    return 0;
}
