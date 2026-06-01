#include <stdio.h>
#include <string.h>
int main() {
    char line[100000] = {0};
    if (!fgets(line, sizeof line, stdin)) line[0] = 0;
    line[strcspn(line, "\r\n")] = 0;
    char words[5000][128]; int cnt = 0;
    char *tok = strtok(line, " ");
    while (tok) { strcpy(words[cnt++], tok); tok = strtok(NULL, " "); }
    for (int i = cnt - 1; i >= 0; i--) {
        fputs(words[i], stdout);
        if (i > 0) putchar(' ');
    }
    putchar('\n');
    return 0;
}
