#include <stdio.h>
#include <string.h>
int main() {
    char s[300];
    fgets(s, 300, stdin); s[strcspn(s, "\n")] = 0;
    int n = strlen(s), ans = -1;
    for (int i = 0; i < n - 1; i++)
        for (int j = n - 1; j > i; j--)
            if (s[i] == s[j]) {
                int len = j - i - 1;
                if (len > ans) ans = len;
                break;
            }
    printf("%d\n", ans);
    return 0;
}
