#include <stdio.h>
#include <string.h>
int main() {
    char s[1001];
    if (!fgets(s, sizeof(s), stdin)) s[0] = '\0';
    else { int n = strlen(s); if (n > 0 && s[n-1] == '\n') s[n-1] = '\0'; }
    int count = 0, in_seg = 0;
    for (int i = 0; s[i]; i++) {
        if (s[i] == ' ') { in_seg = 0; }
        else if (!in_seg) { count++; in_seg = 1; }
    }
    printf("%d\n", count);
    return 0;
}
