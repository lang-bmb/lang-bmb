#include <stdio.h>
#include <string.h>
int main() {
    char a[10000] = {0}, b[10000] = {0};
    if (!fgets(a, sizeof a, stdin)) a[0] = 0;
    if (!fgets(b, sizeof b, stdin)) b[0] = 0;
    a[strcspn(a, "\r\n")] = 0;
    b[strcspn(b, "\r\n")] = 0;
    int i = (int)strlen(a) - 1, j = (int)strlen(b) - 1, carry = 0;
    char out[10010]; int p = 0;
    while (i >= 0 || j >= 0 || carry) {
        int da = i >= 0 ? a[i] - '0' : 0;
        int db = j >= 0 ? b[j] - '0' : 0;
        int sum = da + db + carry;
        out[p++] = (char)('0' + sum % 10);
        carry = sum / 10; i--; j--;
    }
    if (p == 0) out[p++] = '0';
    for (int k = p - 1; k >= 0; k--) putchar(out[k]);
    putchar('\n');
    return 0;
}
