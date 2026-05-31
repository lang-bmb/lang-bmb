#include <stdio.h>
#include <string.h>
int main() {
    char s[15]; scanf("%s", s);
    int n = strlen(s);
    int freq[10] = {0};
    for (int i = 0; i < n; i++) freq[s[i]-'0']++;
    for (int i = 0; i < n; i++) {
        int d = s[i] - '0';
        if (freq[i] != d) { printf("0\n"); return 0; }
    }
    printf("1\n");
    return 0;
}
