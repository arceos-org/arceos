#include <arpa/inet.h>
#include <netdb.h>
#include <netinet/in.h>
#include <stdio.h>
#include <string.h>
#include <sys/socket.h>

const char request[] = "\
GET / HTTP/1.1\r\n\
Host: ident.me\r\n\
Accept: */*\r\n\
\r\n";

int main()
{
    puts("Hello, ArceOS C HTTP client!");
    int sock = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    if (sock == -1) {
        puts("socket() error!");
        return -1;
    }
    struct addrinfo *res;

    if (getaddrinfo("ident.me", NULL, NULL, &res) != 0) {
        puts("getaddrinfo() error!");
        return -1;
    }
    char str[INET_ADDRSTRLEN];
    if (inet_ntop(AF_INET, &(((struct sockaddr_in *)(res->ai_addr))->sin_addr), str,
                  INET_ADDRSTRLEN) == NULL) {
        puts("inet_ntop() error!");
        return -1;
    }
    printf("IP: %s\n", str);
    ((struct sockaddr_in *)(res->ai_addr))->sin_port = htons(80);
    if (connect(sock, res->ai_addr, sizeof(*(res->ai_addr))) != 0) {
        puts("connect() error!");
        return -1;
    }
    char rebuf[2000] = {};
    if (send(sock, request, strlen(request), 0) == -1) {
        puts("send() error!");
        return -1;
    }
    ssize_t l = recv(sock, rebuf, 2000, 0);
    if (l == -1) {
        puts("recv() error!");
        return -1;
    }
    rebuf[l] = '\0';
    printf("%s\n", rebuf);
    return 0;
}
