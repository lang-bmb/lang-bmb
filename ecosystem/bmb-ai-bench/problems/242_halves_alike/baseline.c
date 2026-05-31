#include <stdio.h>
#include <string.h>

int is_vowel(int c) {
    return c=='a'||c=='e'||c=='i'||c=='o'||c=='u'||
           c=='A'||c=='E'||c=='I'||c=='O'||c=='U';
}

int main() {
    char s[2000];
    fgets(s, sizeof(s), stdin);
    int n = (int)strlen(s);
    if (n > 0 && s[n-1] == '\n') s[--n] = '\0';
    int half = n / 2;
    int c1 = 0, c2 = 0;
    for (int i = 0; i < half; i++) if (is_vowel(s[i])) c1++;
    for (int i = half; i < n; i++) if (is_vowel(s[i])) c2++;
    printf("%s\n", c1 == c2 ? "true" : "false");
    return 0;
}
