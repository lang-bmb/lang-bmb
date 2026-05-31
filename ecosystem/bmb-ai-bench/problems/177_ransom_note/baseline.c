#include <stdio.h>
#include <string.h>
int main() {
    char ransom[1001], mag[1001];
    if (!fgets(ransom, sizeof(ransom), stdin)) ransom[0] = '\0';
    else { int n = strlen(ransom); if (n>0 && ransom[n-1]=='\n') ransom[n-1]='\0'; }
    if (!fgets(mag, sizeof(mag), stdin)) mag[0] = '\0';
    else { int n = strlen(mag); if (n>0 && mag[n-1]=='\n') mag[n-1]='\0'; }
    int freq[26] = {0};
    for (int i = 0; mag[i]; i++) if (mag[i]>='a'&&mag[i]<='z') freq[mag[i]-'a']++;
    for (int i = 0; ransom[i]; i++) {
        if (ransom[i]>='a'&&ransom[i]<='z') {
            if (--freq[ransom[i]-'a'] < 0) { printf("0\n"); return 0; }
        }
    }
    printf("1\n");
    return 0;
}
