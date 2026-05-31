#include <stdio.h>
#include <string.h>
int main() {
    char s[10000];
    scanf("%s", s);
    int freq[26] = {0};
    int n = strlen(s);
    for (int i = 0; i < n; i++) freq[s[i]-'a']++;
    int target = -1;
    for (int i = 0; i < 26; i++) {
        if (freq[i] > 0) {
            if (target == -1) target = freq[i];
            else if (freq[i] != target) { printf("0\n"); return 0; }
        }
    }
    printf("1\n");
    return 0;
}
