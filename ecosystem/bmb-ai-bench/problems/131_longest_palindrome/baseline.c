#include <stdio.h>
#include <string.h>

int main() {
    char s[10001];
    if (fgets(s, sizeof(s), stdin)) {
        int len = strlen(s);
        if (len > 0 && s[len-1] == '\n') s[--len] = '\0';
        int freq[128] = {0};
        for (int i = 0; i < len; i++) freq[(unsigned char)s[i]]++;
        int result = 0, has_odd = 0;
        for (int i = 0; i < 128; i++) {
            result += (freq[i] / 2) * 2;
            if (freq[i] % 2) has_odd = 1;
        }
        printf("%d\n", result + has_odd);
    }
    return 0;
}
