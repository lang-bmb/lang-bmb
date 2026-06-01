#include <stdio.h>
#include <string.h>
int main() {
    char s[100000] = {0}, t[100000] = {0};
    if (!fgets(s, sizeof s, stdin)) s[0] = 0;
    if (!fgets(t, sizeof t, stdin)) t[0] = 0;
    s[strcspn(s, "\r\n")] = 0;
    t[strcspn(t, "\r\n")] = 0;
    printf("%d\n", strstr(s, t) != NULL ? 1 : 0);
    return 0;
}
