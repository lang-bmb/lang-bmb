#include <stdio.h>
#include <string.h>
#include <ctype.h>
int is_digit(char c) { return c >= '0' && c <= '9'; }
int parse_num(const char *s, int n, int i) {
    int v = 0;
    while (i < n && is_digit(s[i])) { v = v * 10 + (s[i] - '0'); i++; }
    return v;
}
int main() {
    char s[1000];
    fgets(s, sizeof(s), stdin);
    int n = strlen(s);
    int prev = -1, ok = 1;
    int i = 0;
    while (i < n && ok) {
        if (is_digit(s[i])) {
            int v = parse_num(s, n, i);
            if (v <= prev) ok = 0;
            prev = v;
            while (i < n && is_digit(s[i])) i++;
        } else i++;
    }
    printf("%d\n", ok);
    return 0;
}
