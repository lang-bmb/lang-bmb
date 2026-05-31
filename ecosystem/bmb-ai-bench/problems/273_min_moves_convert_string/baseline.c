#include <stdio.h>
#include <string.h>
int main() {
    char s[1000];
    scanf("%s", s);
    int n = strlen(s), ops = 0, i = 0;
    while (i < n) {
        if (s[i] == 'X') { ops++; i += 3; }
        else i++;
    }
    printf("%d\n", ops);
    return 0;
}
