#include <stdio.h>
int main() {
    int n;
    scanf("%d", &n);
    int arr[1001];
    for (int i = 0; i < n; i++) scanf("%d", &arr[i]);
    int consecutive = 0;
    for (int i = 0; i < n; i++) {
        if (arr[i] % 2 == 1) consecutive++;
        else consecutive = 0;
        if (consecutive >= 3) { printf("1\n"); return 0; }
    }
    printf("0\n");
    return 0;
}
