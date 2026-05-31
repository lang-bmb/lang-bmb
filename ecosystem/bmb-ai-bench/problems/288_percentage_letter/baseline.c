#include <stdio.h>
#include <string.h>
int main() {
    char s[200]; scanf("%s", s);
    char ch[3]; scanf("%s", ch);
    char c = ch[0];
    int n = strlen(s), cnt = 0;
    for (int i = 0; i < n; i++) if (s[i] == c) cnt++;
    printf("%d\n", cnt * 100 / n);
    return 0;
}
