#include <stdio.h>
#include <string.h>
int main() {
    char line[100000] = {0};
    if (!fgets(line, sizeof line, stdin)) line[0] = 0;
    line[strcspn(line, "\r\n")] = 0;
    char words[2000][128]; int freq[2000]; int cnt = 0;
    char *tok = strtok(line, " ");
    while (tok) {
        int idx = -1;
        for (int i = 0; i < cnt; i++) if (strcmp(words[i], tok) == 0) { idx = i; break; }
        if (idx < 0) { strcpy(words[cnt], tok); freq[cnt] = 1; cnt++; }
        else freq[idx]++;
        tok = strtok(NULL, " ");
    }
    int best = 0;
    for (int i = 0; i < cnt; i++) if (freq[i] > best) best = freq[i];
    printf("%d\n", best);
    return 0;
}
