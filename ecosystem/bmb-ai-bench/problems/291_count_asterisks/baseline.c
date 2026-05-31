#include <stdio.h>
#include <string.h>
int main() {
    char s[1000]; scanf("%s", s);
    int n = strlen(s), cnt = 0, in_pair = 0;
    for (int i = 0; i < n; i++) {
        if (s[i] == '|') in_pair = !in_pair;
        else if (s[i] == '*' && !in_pair) cnt++;
    }
    printf("%d\n", cnt);
    return 0;
}
