#include <stdio.h>
#include <string.h>
int main() {
    char s[10000], word[10000], built[10000];
    scanf("%s", s);
    int n; scanf("%d", &n);
    built[0] = '\0';
    for (int i = 0; i < n; i++) {
        scanf("%s", word);
        strcat(built, word);
        if (strcmp(built, s) == 0) { printf("1\n"); return 0; }
        if (strlen(built) >= strlen(s)) break;
    }
    printf("0\n");
    return 0;
}
