#include <stdio.h>
#include <string.h>
int main() {
    int n;
    scanf("%d\n", &n);
    int depth = 0;
    char line[200];
    for (int i = 0; i < n; i++) {
        fgets(line, 200, stdin);
        line[strcspn(line, "\n")] = 0;
        if (strcmp(line, "..") == 0 || strncmp(line, "../", 3) == 0) {
            if (depth > 0) depth--;
        } else if (strcmp(line, ".") == 0 || strncmp(line, "./", 2) == 0) {
            /* stay */
        } else {
            depth++;
        }
    }
    printf("%d\n", depth);
    return 0;
}
