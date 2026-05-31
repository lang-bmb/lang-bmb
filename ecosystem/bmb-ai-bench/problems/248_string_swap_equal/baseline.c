#include <stdio.h>
#include <string.h>
int main() {
    char s1[101], s2[101];
    scanf("%s", s1); scanf("%s", s2);
    int n = strlen(s1);
    if (n != (int)strlen(s2)) { printf("false\n"); return 0; }
    int diff[2], cnt = 0;
    for (int i = 0; i < n; i++) {
        if (s1[i] != s2[i]) {
            if (cnt == 2) { printf("false\n"); return 0; }
            diff[cnt++] = i;
        }
    }
    if (cnt == 0) { printf("true\n"); return 0; }
    if (cnt == 2 && s1[diff[0]] == s2[diff[1]] && s1[diff[1]] == s2[diff[0]]) { printf("true\n"); return 0; }
    printf("false\n");
    return 0;
}
