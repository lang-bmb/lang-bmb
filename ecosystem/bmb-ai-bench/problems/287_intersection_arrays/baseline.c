#include <stdio.h>
#include <string.h>
int main() {
    char s[1000];
    scanf("%s", s);
    int n = strlen(s);
    int best = -1;
    for (int i = 0; i + 2 < n; i++) {
        if (s[i] == s[i+1] && s[i+1] == s[i+2]) {
            int v = s[i] - '0';
            if (v > best) best = v;
        }
    }
    if (best == -1) printf("\n");
    else printf("%d%d%d\n", best, best, best);
    return 0;
}
