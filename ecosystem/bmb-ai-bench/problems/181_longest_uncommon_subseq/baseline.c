#include <stdio.h>
#include <string.h>
int main() {
    char a[201], b[201];
    if (!fgets(a, sizeof(a), stdin)) a[0]='\0'; else { int n=strlen(a); if(n>0&&a[n-1]=='\n') a[n-1]='\0'; }
    if (!fgets(b, sizeof(b), stdin)) b[0]='\0'; else { int n=strlen(b); if(n>0&&b[n-1]=='\n') b[n-1]='\0'; }
    if (strcmp(a, b) == 0) { printf("-1\n"); return 0; }
    int la = strlen(a), lb = strlen(b);
    printf("%d\n", la >= lb ? la : lb);
    return 0;
}
