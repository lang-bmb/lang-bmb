#include <stdio.h>
#include <string.h>
int main() {
    char s[300], out[600] = {0};
    fgets(s, 300, stdin); s[strcspn(s, "\n")] = 0;
    int i = 0, j = 0;
    while (s[i]) {
        if (s[i] == 'G') { out[j++] = 'G'; i++; }
        else if (s[i] == '(' && s[i+1] == ')') { out[j++] = 'o'; i += 2; }
        else { out[j++] = 'a'; out[j++] = 'l'; i += 4; }
    }
    printf("%s\n", out);
    return 0;
}
