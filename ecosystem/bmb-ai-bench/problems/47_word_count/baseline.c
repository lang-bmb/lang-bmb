#include <stdio.h>
#include <string.h>
int main(void) {
    int n; scanf("%d\n", &n);
    char line[4096];
    for (int i = 0; i < n; i++) {
        fgets(line, sizeof(line), stdin);
        int len = strlen(line);
        if (len > 0 && line[len-1] == '\n') line[--len] = 0;
        int count = 0, in_word = 0;
        for (int j = 0; j < len; j++) {
            if (line[j] == ' ') in_word = 0;
            else if (!in_word) { count++; in_word = 1; }
        }
        printf("%d\n", count);
    }
    return 0;
}
