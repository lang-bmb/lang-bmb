#include <stdio.h>
#include <string.h>
int main() {
    char line[100000] = {0};
    if (!fgets(line, sizeof line, stdin)) line[0] = 0;
    line[strcspn(line, "\r\n")] = 0;
    char words[2000][128]; int cnt = 0;
    char *tok = strtok(line, " ");
    while (tok) {
        int found = 0;
        for (int i = 0; i < cnt; i++) if (strcmp(words[i], tok) == 0) { found = 1; break; }
        if (!found) strcpy(words[cnt++], tok);
        tok = strtok(NULL, " ");
    }
    printf("%d\n", cnt);
    return 0;
}
