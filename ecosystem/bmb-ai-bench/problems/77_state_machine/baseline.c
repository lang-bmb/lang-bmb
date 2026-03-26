#include <stdio.h>
int main(void) {
    int n; scanf("%d", &n);
    long long state = 0;
    for (int i = 0; i < n; i++) {
        int cmd; scanf("%d", &cmd);
        if (cmd==1) state++; else if (cmd==2) state--;
        else if (cmd==3) state*=2; else if (cmd==4) state=0;
        else if (cmd==5) state=-state;
    }
    printf("%lld\n", state);
    return 0;
}
