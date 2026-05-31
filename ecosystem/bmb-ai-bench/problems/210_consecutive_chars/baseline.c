#include <stdio.h>
#include <string.h>
int main() {
    char s[1001];
    scanf("%s", s);
    int n = strlen(s);
    int best = 1, cur = 1;
    for (int i = 1; i < n; i++) {
        if (s[i] == s[i-1]) cur++;
        else cur = 1;
        if (cur > best) best = cur;
    }
    printf("%d\n", n == 0 ? 0 : best);
    return 0;
}
