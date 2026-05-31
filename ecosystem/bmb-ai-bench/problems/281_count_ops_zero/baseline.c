#include <stdio.h>
int main() {
    int num1, num2; scanf("%d\n%d", &num1, &num2);
    int ops = 0;
    while (num1 != 0 && num2 != 0) {
        if (num1 >= num2) num1 -= num2;
        else num2 -= num1;
        ops++;
    }
    printf("%d\n", ops);
    return 0;
}
