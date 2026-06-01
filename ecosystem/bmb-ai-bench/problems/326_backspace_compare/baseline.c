#include <stdio.h>
#include <string.h>
static int rebuild(const char *s, char *out) {
    int top = 0;
    for (int i = 0; s[i]; i++) {
        if (s[i] == '#') { if (top > 0) top--; }
        else out[top++] = s[i];
    }
    out[top] = 0;
    return top;
}
int main() {
    char s[100000] = {0}, t[100000] = {0}, a[100000], b[100000];
    if (!fgets(s, sizeof s, stdin)) s[0] = 0;
    if (!fgets(t, sizeof t, stdin)) t[0] = 0;
    s[strcspn(s, "\r\n")] = 0;
    t[strcspn(t, "\r\n")] = 0;
    rebuild(s, a); rebuild(t, b);
    printf("%d\n", strcmp(a, b) == 0 ? 1 : 0);
    return 0;
}
