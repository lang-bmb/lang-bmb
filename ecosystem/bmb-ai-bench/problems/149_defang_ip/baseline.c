#include <stdio.h>
#include <string.h>
int main() {
    char ip[50], buf[200];
    fgets(ip, 50, stdin); ip[strcspn(ip, "\n")] = 0;
    int j = 0;
    for (int i = 0; ip[i]; i++) {
        if (ip[i] == '.') { buf[j++]='['; buf[j++]='.'; buf[j++]=']'; }
        else buf[j++] = ip[i];
    }
    buf[j] = 0;
    printf("%s\n", buf);
    return 0;
}
