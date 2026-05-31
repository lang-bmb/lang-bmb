#include <stdio.h>
#include <string.h>
int has_prefix(const char *word, const char *pref) {
    int pl = strlen(pref);
    return strncmp(word, pref, pl) == 0;
}
int main() {
    int n; scanf("%d ", &n);
    char pref[100]; scanf("%s", pref);
    int cnt = 0;
    char word[1000];
    for (int i = 0; i < n; i++) {
        scanf("%s", word);
        if (has_prefix(word, pref)) cnt++;
    }
    printf("%d\n", cnt);
    return 0;
}
