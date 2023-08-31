
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <arpa/inet.h>
#include <netinet/in.h>
#include <sys/socket.h>

const char header[] = "\
HTTP/1.1 200 OK\r\n\
Content-Type: text/html\r\n\
Content-Length: %u\r\n\
Connection: close\r\n\
\r\n\
";

const char content[] = "<html>\n\
<head>\n\
  <title>Hello, ArceOS</title>\n\
</head>\n\
<body>\n\
  <center>\n\
    <h1>Hello, <a href=\"https://github.com/rcore-os/arceos\">ArceOS</a></h1>\n\
  </center>\n\
  <hr>\n\
  <center>\n\
    <i>Powered by <a href=\"https://github.com/rcore-os/arceos/tree/main/apps/net/httpserver\">ArceOS example HTTP server</a> v0.1.0</i>\n\
  </center>\n\
</body>\n\
</html>\n\
";

int main()
{
    puts("Hello, ArceOS C HTTP server!");
    struct sockaddr_in local, remote;
    int addr_len = sizeof(remote);
    local.sin_family = AF_INET;
    if (inet_pton(AF_INET, "0.0.0.0", &(local.sin_addr)) != 1) {
        perror("inet_pton() error");
        return -1;
    }
    local.sin_port = htons(5555);
    int sock = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    if (sock == -1) {
        perror("socket() error");
        return -1;
    }
    if (bind(sock, (struct sockaddr *)&local, sizeof(local)) != 0) {
        perror("bind() error");
        return -1;
    }
    if (listen(sock, 0) != 0) {
        perror("listen() error");
        return -1;
    }
    puts("listen on: http://0.0.0.0:5555/");
    char buf[1024] = {};
    int client;
    char response[1024] = {};
    snprintf(response, 1024, header, strlen(content));
    strcat(response, content);
    for (;;) {
        client = accept(sock, (struct sockaddr *)&remote, (socklen_t *)&addr_len);
        if (client == -1) {
            perror("accept() error");
            return -1;
        }
        printf("new client %d\n", client);
        if (recv(client, buf, 1024, 0) == -1) {
            perror("recv() error");
            return -1;
        }
        ssize_t l = send(client, response, strlen(response), 0);
        if (l == -1) {
            perror("send() error");
            return -1;
        }
        if (close(client) == -1) {
            perror("close() error");
            return -1;
        }
        printf("client %d close: %ld bytes sent\n", client, l);
    }
    if (close(sock) == -1) {
        perror("close() error");
        return -1;
    }
    return 0;
}
