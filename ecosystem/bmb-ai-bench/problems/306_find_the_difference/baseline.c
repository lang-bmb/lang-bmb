#include <stdio.h>
#include <string.h>
int main() {
    char s[10000] = {0}, t[10000] = {0};
    if (!fgets(s, sizeof s, stdin)) s[0] = 0;
    if (!fgets(t, sizeof t, stdin)) t[0] = 0;
    s[strcspn(s, "\r\n")] = 0;
    t[strcspn(t, "\r\n")] = 0;
    long sum = 0;
    for (int i = 0; s[i]; i++) sum -= (unsigned char)s[i];
    for (int i = 0; t[i]; i++) sum += (unsigned char)t[i];
    printf("%c\n", (int)sum);
    return 0;
}
