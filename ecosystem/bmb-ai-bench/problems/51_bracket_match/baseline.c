#include <stdio.h>
int main(void) {
    int t; scanf("%d", &t);
    while (t--) {
        int n; scanf("%d", &n);
        int stack[1024], top = 0, valid = 1;
        for (int i = 0; i < n; i++) {
            int ch; scanf("%d", &ch);
            if (ch == 40 || ch == 91 || ch == 123) stack[top++] = ch;
            else if (top == 0) valid = 0;
            else {
                int tp = stack[--top];
                if ((ch == 41 && tp != 40) || (ch == 93 && tp != 91) || (ch == 125 && tp != 123))
                    valid = 0;
            }
        }
        if (top != 0) valid = 0;
        printf("%d\n", valid);
    }
    return 0;
}
