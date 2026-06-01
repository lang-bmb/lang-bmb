#include <stdio.h>
#include <string.h>
#include <ctype.h>
int main() {
    char s[100000] = {0};
    if (!fgets(s, sizeof s, stdin)) s[0] = 0;
    s[strcspn(s, "\r\n")] = 0;
    int i = 0, j = (int)strlen(s) - 1, ok = 1;
    while (i < j) {
        if (!isalnum((unsigned char)s[i])) { i++; continue; }
        if (!isalnum((unsigned char)s[j])) { j--; continue; }
        if (tolower((unsigned char)s[i]) != tolower((unsigned char)s[j])) { ok = 0; break; }
        i++; j--;
    }
    printf("%d\n", ok);
    return 0;
}
