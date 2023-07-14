#include <stdio.h>
#include <string.h>
#include <arpa/inet.h>
#include <netinet/in.h>
#include <sys/socket.h>

const char res_suffix[11] = "_response\n";

int main()
{
    puts("Hello, ArceOS C UDP server!");
    struct sockaddr_in local, remote;
    int addr_len = sizeof(remote);
    local.sin_family = AF_INET;
    if (inet_pton(AF_INET, "0.0.0.0", &(local.sin_addr)) != 1) {
        perror("inet_pton() error");
        return -1;
    }
    local.sin_port = htons(5555);
    int sock = socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP);
    if (sock == -1) {
        perror("socket() error");
        return -1;
    }
    if (bind(sock, (struct sockaddr *)&local, sizeof(local)) != 0) {
        perror("bind() error");
        return -1;
    }
    puts("listen on: 0.0.0.0:5555");
    char buf[1024] = {};
    for (;;) {
        ssize_t l =
            recvfrom(sock, buf, 1024, 0, (struct sockaddr *)&remote, (socklen_t *)&addr_len);
        if (l == -1) {
            perror("recvfrom() error");
            return -1;
        }
        uint8_t *addr = (uint8_t *)&(remote.sin_addr);
        printf("recv: %d Bytes from %d.%d.%d.%d:%d\n", l, addr[0], addr[1], addr[2], addr[3],
               ntohs(remote.sin_port));
        buf[l] = '\0';
        printf("%s\n", buf);
        if (l > 1024 - 10) {
            puts("received message too long");
            return 0;
        }
        strncpy(buf + l - 1, res_suffix, 10);
        if (sendto(sock, buf, l + 10, 0, (struct sockaddr *)&remote, addr_len) == -1) {
            perror("sendto() error");
            return -1;
        }
    }
    return 0;
}
