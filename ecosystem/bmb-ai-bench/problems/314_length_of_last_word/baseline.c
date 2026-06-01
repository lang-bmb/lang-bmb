#include <stdio.h>
#include <string.h>
int main() {
    char s[100000] = {0};
    if (!fgets(s, sizeof s, stdin)) s[0] = 0;
    s[strcspn(s, "\r\n")] = 0;
    int i = (int)strlen(s) - 1;
    while (i >= 0 && s[i] == ' ') i--;
    int len = 0;
    while (i >= 0 && s[i] != ' ') { len++; i--; }
    printf("%d\n", len);
    return 0;
}
