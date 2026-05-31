#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    int a[101];
    for (int i = 0; i < n; i++) scanf("%d", &a[i]);
    int drops = 0;
    for (int i = 0; i < n; i++) {
        if (a[i] > a[(i+1)%n]) drops++;
    }
    printf("%s\n", drops <= 1 ? "true" : "false");
    return 0;
}
