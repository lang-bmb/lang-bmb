#include <stdio.h>
#include <string.h>
int main() {
    int n;
    scanf("%d", &n);
    int times[300];
    for (int i = 0; i < n; i++) scanf("%d", &times[i]);
    scanf("\n");
    char keys[301];
    fgets(keys, 301, stdin); keys[strcspn(keys, "\n")] = 0;
    int max_dur = 0; char best = 'a' - 1;
    for (int i = 0; i < n; i++) {
        int prev = (i == 0) ? 0 : times[i-1];
        int dur = times[i] - prev;
        char c = keys[i];
        if (dur > max_dur || (dur == max_dur && c > best)) {
            max_dur = dur; best = c;
        }
    }
    printf("%c\n", best);
    return 0;
}
