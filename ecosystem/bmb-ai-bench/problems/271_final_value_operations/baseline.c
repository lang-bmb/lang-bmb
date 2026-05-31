#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    char op[5];
    int x = 0;
    for (int i = 0; i < n; i++) {
        scanf("%s", op);
        if (op[1] == '+') x++;
        else x--;
    }
    printf("%d\n", x);
    return 0;
}
