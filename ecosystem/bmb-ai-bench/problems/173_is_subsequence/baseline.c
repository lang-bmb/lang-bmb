#include <stdio.h>
#include <string.h>
int is_subseq(const char *s, const char *t, int i, int j) {
    if (s[i] == '\0') return 1;
    if (t[j] == '\0') return 0;
    if (s[i] == t[j]) return is_subseq(s, t, i+1, j+1);
    return is_subseq(s, t, i, j+1);
}
int main() {
    char s[1001], t[1001];
    if (!fgets(s, sizeof(s), stdin)) s[0] = '\0';
    else { int n = strlen(s); if (n > 0 && s[n-1] == '\n') s[n-1] = '\0'; }
    if (!fgets(t, sizeof(t), stdin)) t[0] = '\0';
    else { int n = strlen(t); if (n > 0 && t[n-1] == '\n') t[n-1] = '\0'; }
    printf("%d\n", is_subseq(s, t, 0, 0));
    return 0;
}
