#include <stdio.h>
#include <string.h>
int main() {
    char s[300];
    fgets(s, 300, stdin); s[strcspn(s, "\n")] = 0;
    int n = strlen(s);
    char result[300] = {0};
    for (int i = 0; i < n; i++) {
        int idx;
        scanf("%d", &idx);
        result[idx] = s[i];
    }
    result[n] = 0;
    printf("%s\n", result);
    return 0;
}
