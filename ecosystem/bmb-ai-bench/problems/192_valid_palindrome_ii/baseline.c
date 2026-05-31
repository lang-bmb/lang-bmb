#include <stdio.h>
#include <string.h>
int is_pal(char *s, int lo, int hi) {
    while (lo < hi) {
        if (s[lo] != s[hi]) return 0;
        lo++; hi--;
    }
    return 1;
}
int main() {
    char s[100001];
    if (!fgets(s, sizeof(s), stdin)) s[0]='\0';
    int n = strlen(s);
    if (n > 0 && s[n-1] == '\n') { s[n-1] = '\0'; n--; }
    int lo = 0, hi = n-1;
    while (lo < hi) {
        if (s[lo] != s[hi]) {
            printf("%d\n", is_pal(s,lo+1,hi) || is_pal(s,lo,hi-1) ? 1 : 0);
            return 0;
        }
        lo++; hi--;
    }
    printf("1\n");
    return 0;
}
