#include <stdio.h>
#include <string.h>

int main() {
    char s[10];
    scanf("%s", s);
    long long result = 0;
    for (int i = 0; s[i]; i++) {
        result = result * 26 + (s[i] - 'A' + 1);
    }
    printf("%lld\n", result);
    return 0;
}
