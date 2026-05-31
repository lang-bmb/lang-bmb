#include <stdio.h>
#include <string.h>
int main() {
    char s[1000];
    scanf("%s", s);
    int depth = 0, max_d = 0;
    for (int i = 0; i < (int)strlen(s); i++) {
        if (s[i] == '(') { depth++; if (depth > max_d) max_d = depth; }
        else if (s[i] == ')') depth--;
    }
    printf("%d\n", max_d);
    return 0;
}
