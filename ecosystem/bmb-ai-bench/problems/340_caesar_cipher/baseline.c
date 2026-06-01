#include <stdio.h>
#include <string.h>
int main() {
    char s[100000] = {0}; long long k;
    if (!fgets(s, sizeof s, stdin)) s[0] = 0;
    s[strcspn(s, "\r\n")] = 0;
    scanf("%lld", &k);
    for (int i = 0; s[i]; i++) {
        unsigned char b = s[i];
        if (b >= 'a' && b <= 'z') putchar('a' + (int)((b - 'a' + k) % 26));
        else if (b >= 'A' && b <= 'Z') putchar('A' + (int)((b - 'A' + k) % 26));
        else putchar(b);
    }
    putchar('\n');
    return 0;
}
