#include <stdio.h>
#include <string.h>
void rev(char *s, int lo, int hi) {
    while (lo < hi) { char t = s[lo]; s[lo] = s[hi]; s[hi] = t; lo++; hi--; }
}
int main() {
    char s[1001];
    if (!fgets(s, sizeof(s), stdin)) s[0]='\0';
    int n = strlen(s);
    if (n>0 && s[n-1]=='\n') { s[n-1]='\0'; n--; }
    int start = 0;
    for (int i = 0; i <= n; i++) {
        if (s[i]==' ' || s[i]=='\0') { rev(s, start, i-1); start = i+1; }
    }
    printf("%s\n", s);
    return 0;
}
