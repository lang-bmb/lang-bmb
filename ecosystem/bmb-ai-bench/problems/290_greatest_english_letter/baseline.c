#include <stdio.h>
#include <string.h>
int main() {
    char s[1000]; scanf("%s", s);
    int n = strlen(s);
    int has_upper[26] = {0}, has_lower[26] = {0};
    for (int i = 0; i < n; i++) {
        if (s[i] >= 'A' && s[i] <= 'Z') has_upper[s[i]-'A'] = 1;
        if (s[i] >= 'a' && s[i] <= 'z') has_lower[s[i]-'a'] = 1;
    }
    for (int i = 25; i >= 0; i--) {
        if (has_upper[i] && has_lower[i]) { printf("%c\n", 'A'+i); return 0; }
    }
    printf("\n");
    return 0;
}
