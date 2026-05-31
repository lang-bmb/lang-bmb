#include <stdio.h>
int main() {
    char s[20];
    scanf("%s", s);
    char c1 = s[0], c2 = s[3];
    int r1 = (s[1] - '0'), r2;
    if (s[4] == '\0') { r2 = s[4-1] - '0'; }
    int n1, n2;
    sscanf(s, "%c%d:%c%d", &c1, &n1, &c2, &n2);
    int first = 1;
    for (char c = c1; c <= c2; c++) {
        for (int r = n1; r <= n2; r++) {
            if (!first) printf(" ");
            printf("%c%d", c, r);
            first = 0;
        }
    }
    printf("\n");
    return 0;
}
