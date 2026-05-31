#include <stdio.h>
#include <string.h>

int main() {
    char s[10001];
    if (fgets(s, sizeof(s), stdin)) {
        int len = strlen(s);
        if (len > 0 && s[len-1] == '\n') s[--len] = '\0';
        for (int i = 0, j = len-1; i < j; i++, j--) {
            char tmp = s[i]; s[i] = s[j]; s[j] = tmp;
        }
        printf("%s\n", s);
    }
    return 0;
}
