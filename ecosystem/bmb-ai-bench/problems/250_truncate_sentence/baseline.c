#include <stdio.h>
#include <string.h>
int main() {
    char s[1001]; int k;
    fgets(s, sizeof(s), stdin);
    int n = strlen(s);
    if (n > 0 && s[n-1] == '\n') s[--n] = '\0';
    scanf("%d", &k);
    int words = 0, i;
    for (i = 0; i < n; i++) {
        if (s[i] == ' ') {
            words++;
            if (words == k) { s[i] = '\0'; break; }
        }
    }
    printf("%s\n", s);
    return 0;
}
