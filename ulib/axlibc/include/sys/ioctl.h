#ifndef __SYS_IOCTL_H__
#define __SYS_IOCTL_H__

#define TCGETS       0x5401
#define TCSETS       0x5402
#define TCSETSW      0x5403
#define TCSETSF      0x5404
#define TCGETA       0x5405
#define TCSETA       0x5406
#define TCSETAW      0x5407
#define TCSETAF      0x5408
#define TCSBRK       0x5409
#define TCXONC       0x540A
#define TCFLSH       0x540B
#define TIOCEXCL     0x540C
#define TIOCNXCL     0x540D
#define TIOCSCTTY    0x540E
#define TIOCGPGRP    0x540F
#define TIOCSPGRP    0x5410
#define TIOCOUTQ     0x5411
#define TIOCSTI      0x5412
#define TIOCGWINSZ   0x5413
#define TIOCSWINSZ   0x5414
#define TIOCMGET     0x5415
#define TIOCMBIS     0x5416
#define TIOCMBIC     0x5417
#define TIOCMSET     0x5418
#define TIOCGSOFTCAR 0x5419
#define TIOCSSOFTCAR 0x541A
#define FIONREAD     0x541B
#define TIOCINQ      FIONREAD
#define TIOCLINUX    0x541C
#define TIOCCONS     0x541D
#define TIOCGSERIAL  0x541E
#define TIOCSSERIAL  0x541F
#define TIOCPKT      0x5420
#define FIONBIO      0x5421
#define TIOCNOTTY    0x5422
#define TIOCSETD     0x5423
#define TIOCGETD     0x5424
#define TCSBRKP      0x5425
#define TIOCSBRK     0x5427
#define TIOCCBRK     0x5428
#define TIOCGSID     0x5429
#define TIOCGRS485   0x542E
#define TIOCSRS485   0x542F
#define TIOCGPTN     0x80045430
#define TIOCSPTLCK   0x40045431
#define TIOCGDEV     0x80045432
#define TCGETX       0x5432
#define TCSETX       0x5433
#define TCSETXF      0x5434
#define TCSETXW      0x5435
#define TIOCSIG      0x40045436
#define TIOCVHANGUP  0x5437
#define TIOCGPKT     0x80045438
#define TIOCGPTLCK   0x80045439
#define TIOCGEXCL    0x80045440
#define TIOCGPTPEER  0x5441
#define TIOCGISO7816 0x80285442
#define TIOCSISO7816 0xc0285443

int ioctl(int, int, ...);

#endif // __SYS_IOCTL_H__
