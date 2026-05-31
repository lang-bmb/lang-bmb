#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#define MAXN 200
#define MAXLEN 200
char froms[MAXN][MAXLEN];
char tos[MAXN][MAXLEN];
int in_from(const char *city, int n) {
    for (int i = 0; i < n; i++) if (strcmp(froms[i], city) == 0) return 1;
    return 0;
}
int main() {
    int n;
    scanf("%d\n", &n);
    for (int i = 0; i < n; i++) {
        fgets(froms[i], MAXLEN, stdin); froms[i][strcspn(froms[i], "\n")] = 0;
        fgets(tos[i], MAXLEN, stdin); tos[i][strcspn(tos[i], "\n")] = 0;
    }
    for (int i = 0; i < n; i++)
        if (!in_from(tos[i], n)) { printf("%s\n", tos[i]); return 0; }
    return 0;
}
