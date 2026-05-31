#include <stdio.h>
#include <string.h>
#include <stdlib.h>
int main() {
    int n;
    scanf("%d\n", &n);
    long long stack[1001];
    int top = 0;
    for (int i = 0; i < n; i++) {
        char op[20];
        fgets(op, sizeof(op), stdin);
        int len = strlen(op);
        if (len > 0 && op[len-1] == '\n') op[len-1] = '\0';
        if (strcmp(op, "C") == 0) { if (top > 0) top--; }
        else if (strcmp(op, "D") == 0) { stack[top] = stack[top-1]*2; top++; }
        else if (strcmp(op, "+") == 0) { stack[top] = stack[top-1]+stack[top-2]; top++; }
        else { stack[top++] = atoll(op); }
    }
    long long sum = 0;
    for (int i = 0; i < top; i++) sum += stack[i];
    printf("%lld\n", sum);
    return 0;
}
