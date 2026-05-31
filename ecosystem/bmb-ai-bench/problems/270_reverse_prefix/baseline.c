#include <stdio.h>
#include <string.h>
int main() {
    char word[200], ch[3];
    scanf("%s", word);
    scanf("%s", ch);
    char c = ch[0];
    int n = strlen(word);
    int k = -1;
    for (int i = 0; i < n; i++) if (word[i] == c) { k = i; break; }
    if (k >= 0) {
        for (int l = 0, r = k; l < r; l++, r--) {
            char tmp = word[l]; word[l] = word[r]; word[r] = tmp;
        }
    }
    printf("%s\n", word);
    return 0;
}
