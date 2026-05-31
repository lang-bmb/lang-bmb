#include <stdio.h>
int main() {
    int root, left, right;
    scanf("%d\n%d\n%d", &root, &left, &right);
    printf("%d\n", root == left + right ? 1 : 0);
    return 0;
}
