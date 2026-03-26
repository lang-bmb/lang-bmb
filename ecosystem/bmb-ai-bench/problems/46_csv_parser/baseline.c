#include <stdio.h>
#include <string.h>
int main(void) {
    int n; scanf("%d\n", &n);
    char line[4096];
    for (int i = 0; i < n; i++) {
        fgets(line, sizeof(line), stdin);
        int len = strlen(line);
        if (len > 0 && line[len-1] == '\n') line[--len] = 0;
        if (len == 0) { printf("0\n"); continue; }
        int count = 1;
        for (int j = 0; j < len; j++) if (line[j] == ',') count++;
        printf("%d\n", count);
    }
    return 0;
}
