#include <stdio.h>
#include <string.h>
long long gcd(long long a, long long b) { return b == 0 ? a : gcd(b, a % b); }
int main() {
    char s[200], t[200];
    fgets(s, 200, stdin); s[strcspn(s, "\n")] = 0;
    fgets(t, 200, stdin); t[strcspn(t, "\n")] = 0;
    char st[400], ts[400];
    snprintf(st, 400, "%s%s", s, t);
    snprintf(ts, 400, "%s%s", t, s);
    if (strcmp(st, ts) == 0) {
        long long g = gcd(strlen(s), strlen(t));
        char res[200]; strncpy(res, s, g); res[g] = 0;
        printf("%s\n", res);
    } else printf("\n");
    return 0;
}
