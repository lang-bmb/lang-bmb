#include <stdio.h>
#include <string.h>
int main() {
    char s[100000] = {0};
    if (!fgets(s, sizeof s, stdin)) s[0] = 0;
    s[strcspn(s, "\r\n")] = 0;
    char stk[100000]; int top = 0, ok = 1;
    for (int i = 0; s[i]; i++) {
        char c = s[i];
        if (c == '(' || c == '[' || c == '{') stk[top++] = c;
        else {
            if (top == 0) { ok = 0; break; }
            char o = stk[--top];
            if (!((o=='('&&c==')')||(o=='['&&c==']')||(o=='{'&&c=='}'))) { ok = 0; break; }
        }
    }
    if (top != 0) ok = 0;
    printf("%d\n", ok);
    return 0;
}
