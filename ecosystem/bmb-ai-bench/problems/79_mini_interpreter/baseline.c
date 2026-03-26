#include <stdio.h>
int main(void) {
    int n; scanf("%d", &n);
    long long stack[1024]; int top = 0;
    for (int i = 0; i < n; i++) {
        int op; scanf("%d", &op);
        if (op == 1) { long long x; scanf("%lld", &x); stack[top++] = x; }
        else if (op == 2) { long long b=stack[--top],a=stack[--top]; stack[top++]=a+b; }
        else if (op == 3) { long long b=stack[--top],a=stack[--top]; stack[top++]=a-b; }
        else if (op == 4) { long long b=stack[--top],a=stack[--top]; stack[top++]=a*b; }
        else if (op == 5) { stack[top] = stack[top-1]; top++; }
        else if (op == 6) printf("%lld\n", stack[top-1]);
    }
    return 0;
}
