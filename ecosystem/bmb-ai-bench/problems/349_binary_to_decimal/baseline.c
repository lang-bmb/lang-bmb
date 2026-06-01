#include <stdio.h>
#include <string.h>
int main() {
    char s[100000] = {0};
    if (!fgets(s, sizeof s, stdin)) s[0] = 0;
    s[strcspn(s, "\r\n")] = 0;
    long long acc = 0;
    for (int i = 0; s[i]; i++) acc = acc * 2 + (s[i] - '0');
    printf("%lld\n", acc);
    return 0;
}
