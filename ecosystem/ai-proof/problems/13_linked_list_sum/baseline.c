#include <stdio.h>
#include <stdlib.h>

struct Node {
    long long val;
    struct Node *next;
};

int main(void) {
    int n;
    scanf("%d", &n);
    struct Node *head = NULL, *tail = NULL;
    for (int i = 0; i < n; i++) {
        struct Node *node = (struct Node *)malloc(sizeof(struct Node));
        scanf("%lld", &node->val);
        node->next = NULL;
        if (!head) head = tail = node;
        else { tail->next = node; tail = node; }
    }
    long long sum = 0;
    for (struct Node *cur = head; cur; cur = cur->next)
        sum += cur->val;
    printf("%lld\n", sum);
    while (head) { struct Node *t = head; head = head->next; free(t); }
    return 0;
}
