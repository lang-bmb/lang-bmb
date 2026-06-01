#include <stdio.h>
#include <string.h>
int main() {
    char s[100000] = {0};
    if (!fgets(s, sizeof s, stdin)) s[0] = 0;
    s[strcspn(s, "\r\n")] = 0;
    int open = 0, ans = 0;
    for (int i = 0; s[i]; i++) {
        if (s[i] == '(') open++;
        else { if (open > 0) open--; else ans++; }
    }
    printf("%d\n", ans + open);
    return 0;
}
