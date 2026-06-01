#include <stdio.h>
#include <string.h>
static int val(char c){switch(c){case 'I':return 1;case 'V':return 5;case 'X':return 10;case 'L':return 50;case 'C':return 100;case 'D':return 500;default:return 1000;}}
int main() {
    char s[100] = {0};
    if (!fgets(s, sizeof s, stdin)) s[0] = 0;
    s[strcspn(s, "\r\n")] = 0;
    int n = (int)strlen(s), acc = 0;
    for (int i = 0; i < n; i++) {
        int v = val(s[i]);
        if (i + 1 < n && v < val(s[i+1])) acc -= v; else acc += v;
    }
    printf("%d\n", acc);
    return 0;
}
