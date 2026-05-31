#include <stdio.h>
int main() {
    int n, first;
    scanf("%d", &n);
    scanf("%d", &first);
    int encoded[1000], arr[1001];
    for (int i = 0; i < n; i++) scanf("%d", &encoded[i]);
    arr[0] = first;
    for (int i = 0; i < n; i++) arr[i+1] = encoded[i] ^ arr[i];
    for (int i = 0; i <= n; i++) {
        if (i > 0) printf(" ");
        printf("%d", arr[i]);
    }
    printf("\n");
    return 0;
}
