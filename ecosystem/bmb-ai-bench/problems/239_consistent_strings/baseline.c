#include <stdio.h>
#include <string.h>
int main() {
    char allowed[30];
    fgets(allowed, 30, stdin); allowed[strcspn(allowed, "\n")] = 0;
    int n; scanf("%d\n", &n);
    int cnt = 0;
    for (int i = 0; i < n; i++) {
        char word[100];
        fgets(word, 100, stdin); word[strcspn(word, "\n")] = 0;
        int ok = 1;
        for (int j = 0; word[j]; j++) {
            if (!strchr(allowed, word[j])) { ok = 0; break; }
        }
        if (ok) cnt++;
    }
    printf("%d\n", cnt);
    return 0;
}
