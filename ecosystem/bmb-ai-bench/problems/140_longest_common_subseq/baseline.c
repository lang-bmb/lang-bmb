#include <stdio.h>
#include <string.h>

int main() {
    char s[10001];
    if (fgets(s, sizeof(s), stdin)) {
        int len = strlen(s);
        if (len > 0 && s[len-1] == '\n') s[--len] = '\0';
        int freq[128] = {0};
        for (int i = 0; i < len; i++) freq[(unsigned char)s[i]]++;
        for (int i = 0; i < len; i++) {
            if (freq[(unsigned char)s[i]] == 1) { printf("%d\n", i); return 0; }
        }
        printf("-1\n");
    }
    return 0;
}
