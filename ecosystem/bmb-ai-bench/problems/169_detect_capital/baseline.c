#include <stdio.h>
#include <ctype.h>
#include <string.h>
int main() {
    char s[1001]; scanf("%s", s);
    int n = strlen(s), up = 0;
    for (int i = 0; i < n; i++) if (isupper(s[i])) up++;
    int ok = (up == n) || (up == 0) || (up == 1 && isupper(s[0]));
    printf("%d\n", ok ? 1 : 0);
    return 0;
}
