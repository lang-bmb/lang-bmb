#include <stdio.h>
#include <string.h>
int main() {
    char s[200]; scanf("%s", s);
    int seen[26] = {0};
    for (int i = 0; s[i]; i++) {
        int c = s[i] - 'a';
        if (seen[c]) { printf("%c\n", s[i]); return 0; }
        seen[c] = 1;
    }
    printf("\n");
    return 0;
}
