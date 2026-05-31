#include <stdio.h>
int main() {
    int n;
    scanf("%d", &n);
    int arr[10001];
    for (int i = 0; i < n; i++) scanf("%d", &arr[i]);
    int carry = 1;
    for (int i = n - 1; i >= 0; i--) {
        int sum = arr[i] + carry;
        arr[i] = sum % 10;
        carry = sum / 10;
    }
    if (carry) printf("1\n");
    for (int i = 0; i < n; i++) printf("%d\n", arr[i]);
    return 0;
}
