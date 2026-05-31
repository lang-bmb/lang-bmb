#include <stdio.h>
#include <string.h>
int is_palindrome(const char *s, int n) {
    for (int i = 0; i < n/2; i++)
        if (s[i] != s[n-1-i]) return 0;
    return 1;
}
int main() {
    int n; scanf("%d", &n);
    char word[1000];
    for (int i = 0; i < n; i++) {
        scanf("%s", word);
        if (is_palindrome(word, strlen(word))) { printf("%s\n", word); return 0; }
    }
    printf("\n");
    return 0;
}
