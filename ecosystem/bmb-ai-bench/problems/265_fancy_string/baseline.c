#include <stdio.h>
#include <string.h>
int main() {
    char s[100005], out[100005];
    scanf("%s", s);
    int n = strlen(s), m = 0;
    for (int i = 0; i < n; i++) {
        if (m >= 2 && out[m-1] == s[i] && out[m-2] == s[i]) continue;
        out[m++] = s[i];
    }
    out[m] = '\0';
    printf("%s\n", out);
    return 0;
}
