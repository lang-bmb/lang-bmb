#include <stdio.h>
#include <string.h>
int main() {
    char s[10001], t[10001];
    scanf("%s", s);
    scanf("%s", t);
    if (strlen(s) != strlen(t)) { printf("false\n"); return 0; }
    int freq[26] = {0};
    for (int i = 0; s[i]; i++) freq[(int)(s[i] - 'a')]++;
    for (int i = 0; t[i]; i++) freq[(int)(t[i] - 'a')]--;
    for (int i = 0; i < 26; i++) {
        if (freq[i] != 0) { printf("false\n"); return 0; }
    }
    printf("true\n");
    return 0;
}
