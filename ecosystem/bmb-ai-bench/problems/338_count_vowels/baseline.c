#include <stdio.h>
#include <string.h>
#include <ctype.h>
int main() {
    char s[100000] = {0};
    if (!fgets(s, sizeof s, stdin)) s[0] = 0;
    s[strcspn(s, "\r\n")] = 0;
    int cnt = 0;
    for (int i = 0; s[i]; i++) {
        char c = tolower((unsigned char)s[i]);
        if (c=='a'||c=='e'||c=='i'||c=='o'||c=='u') cnt++;
    }
    printf("%d\n", cnt);
    return 0;
}
