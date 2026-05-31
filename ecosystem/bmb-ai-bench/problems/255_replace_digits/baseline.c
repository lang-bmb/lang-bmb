#include <stdio.h>
#include <string.h>
int main() {
    char s[101];
    scanf("%s", s);
    int n = strlen(s);
    for (int i = 1; i < n; i += 2) {
        s[i] = s[i-1] + (s[i] - '0');
    }
    printf("%s\n", s);
    return 0;
}
