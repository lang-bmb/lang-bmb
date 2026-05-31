#include <stdio.h>
int main() {
    int n;
    scanf("%d", &n);
    int steps = 0;
    while (n > 0) {
        if (n % 2 == 0) n /= 2;
        else n -= 1;
        steps++;
    }
    printf("%d\n", steps);
    return 0;
}
