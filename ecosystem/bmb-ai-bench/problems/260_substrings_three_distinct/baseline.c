#include <stdio.h>
#include <string.h>
int main() {
    char s[200];
    scanf("%s", s);
    int n = strlen(s), cnt = 0;
    for (int i = 0; i + 2 < n; i++) {
        if (s[i] != s[i+1] && s[i+1] != s[i+2] && s[i] != s[i+2]) cnt++;
    }
    printf("%d\n", cnt);
    return 0;
}
