#include <stdio.h>
#include <string.h>
int main() {
    char s[10000] = {0}, t[10000] = {0};
    if (!fgets(s, sizeof s, stdin)) s[0] = 0;
    if (!fgets(t, sizeof t, stdin)) t[0] = 0;
    s[strcspn(s, "\r\n")] = 0;
    t[strcspn(t, "\r\n")] = 0;
    int fs[256] = {0}, ft[256] = {0};
    for (int i = 0; s[i]; i++) fs[(unsigned char)s[i]]++;
    for (int i = 0; t[i]; i++) ft[(unsigned char)t[i]]++;
    int ok = 1;
    for (int c = 0; c < 256; c++) if (fs[c] != ft[c]) ok = 0;
    printf("%d\n", ok);
    return 0;
}
