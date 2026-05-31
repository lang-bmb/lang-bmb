#include <stdio.h>
#include <string.h>
int count_words(const char *line, int len) {
    if (len == 0 || line[0] == '\n') return 0;
    int words = 1;
    for (int j = 0; j < len; j++) if (line[j] == ' ') words++;
    return words;
}
int main() {
    int n; scanf("%d ", &n);
    int max_words = 0;
    char line[1000];
    for (int i = 0; i < n; i++) {
        fgets(line, sizeof(line), stdin);
        int len = strlen(line);
        while (len > 0 && (line[len-1] == '\n' || line[len-1] == '\r')) len--;
        line[len] = '\0';
        int w = count_words(line, len);
        if (w > max_words) max_words = w;
    }
    printf("%d\n", max_words);
    return 0;
}
