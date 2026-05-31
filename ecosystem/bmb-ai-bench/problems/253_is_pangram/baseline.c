#include <stdio.h>
#include <string.h>
int main() {
    char s[1001];
    fgets(s, sizeof(s), stdin);
    int seen[26] = {0};
    for (int i = 0; s[i]; i++) {
        if (s[i] >= 'a' && s[i] <= 'z') seen[s[i]-'a'] = 1;
    }
    int ok = 1;
    for (int i = 0; i < 26; i++) if (!seen[i]) { ok = 0; break; }
    printf("%s\n", ok ? "true" : "false");
    return 0;
}
