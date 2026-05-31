#include <stdio.h>
#include <string.h>
int main() {
    char w1[101], w2[101];
    scanf("%s", w1); scanf("%s", w2);
    int n1 = strlen(w1), n2 = strlen(w2);
    int i = 0, j = 0;
    while (i < n1 || j < n2) {
        if (i < n1) putchar(w1[i++]);
        if (j < n2) putchar(w2[j++]);
    }
    putchar('\n');
    return 0;
}
